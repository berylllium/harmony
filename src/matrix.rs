pub mod session;

use std::{assert_matches, path::PathBuf};

use gpui::*;
use matrix_sdk::{
    AuthSession, Client, ClientBuildError,
    authentication::matrix::MatrixSession,
    config::SyncSettings,
    ruma::{
        OwnedUserId,
        api::client::{self, sync::sync_events::v5::response::Room},
        events::room::message::SyncRoomMessageEvent,
    },
};
use rand::distr::{Alphanumeric, SampleString};

use crate::{
    environment,
    matrix::session::{SessionMetadata, SessionSecrets},
    tokio_bridge,
};

const USER_DB_DIR_NAME: &'static str = "user_db";
const USER_DB_PASSWORD_LEN: usize = 20;

#[derive(Debug)]
pub enum ConnectionState {
    CheckingForSession,
    AwaitingLogin,
    Connecting,
    Connected(Client),
    Error(AuthError),
}

#[derive(Clone)]
pub enum AuthInfo {
    /// Authenticate using an existing session.
    Session,
    /// Authenticate using username and password.
    Password {
        homeserver: String,
        username: String,
        password: String,
    },
}

pub struct Matrix {
    pub connection: ConnectionState,
}

impl Matrix {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mut s = Self {
            connection: ConnectionState::CheckingForSession,
        };

        // Immediately attempt a session auth.
        s.auth(cx, AuthInfo::Session).detach();
        s
    }

    pub fn entity(cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(cx))
    }

    /// Attempt authenticating with the homeserver in a background thread.
    pub fn auth(&mut self, cx: &mut Context<Self>, auth_info: AuthInfo) -> Task<()> {
        cx.spawn(async move |s, cx| {
            let client_auth_info = auth_info.clone();
            let client = tokio_bridge::spawn_async(cx, async move {
                match client_auth_info {
                    AuthInfo::Session => Self::auth_session().await,
                    AuthInfo::Password {
                        homeserver,
                        username,
                        password,
                    } => Self::auth_password(homeserver, username, password).await,
                }
            })
            .await
            .unwrap();

            s.update(cx, |s, cx| {
                s.connection = match client {
                    Ok(client) => {
                        // Start sync on successful connection.
                        tokio_bridge::spawn(cx, Self::sync(client.clone())).detach();

                        ConnectionState::Connected(client)
                    }
                    Err(e) => match auth_info {
                        AuthInfo::Session => {
                            tracing::warn!(
                                "Error during session authentication, skipping auth: {}",
                                e
                            );
                            ConnectionState::AwaitingLogin
                        }
                        AuthInfo::Password { .. } => ConnectionState::Error(e),
                    },
                };

                cx.notify();
            })
            .unwrap();
        })
    }
}

impl Matrix {
    async fn auth_session() -> Result<Client, AuthError> {
        let Some(metadata) = SessionMetadata::load().await.unwrap() else {
            return Err(AuthError::NoSession);
        };

        let Ok(Some(secrets)) = SessionSecrets::load(&metadata.user_id) else {
            return Err(AuthError::NoSession);
        };

        let client = Client::builder()
            .homeserver_url(metadata.homeserver)
            .sqlite_store(metadata.db_path, Some(&secrets.db_passphrase))
            .handle_refresh_tokens()
            .build()
            .await
            .map_err(|e| AuthError::ConnectionError(e.to_string()))?;

        client
            .restore_session(MatrixSession {
                meta: matrix_sdk::SessionMeta {
                    user_id: metadata
                        .user_id
                        .parse::<OwnedUserId>()
                        .map_err(|err| AuthError::InvalidUserId(err.to_string()))?,
                    device_id: secrets.device_id.into(),
                },
                tokens: matrix_sdk::SessionTokens {
                    access_token: secrets.access_token,
                    refresh_token: secrets.refersh_token,
                },
            })
            .await
            .map_err(|e| AuthError::InvalidSession(e.to_string()))?;

        Ok(client)
    }

    async fn auth_password(
        homeserver: String,
        username: String,
        password: String,
    ) -> Result<Client, AuthError> {
        let db_path = db_path_for_user(&username);
        let db_passphrase = if db_path.exists() {
            // A user database already exists. Try restoring session metadata.
            let secrets = SessionMetadata::load()
                .await
                .ok()
                .flatten()
                .map(|m| SessionSecrets::load(&m.user_id))
                .transpose()
                .ok()
                .flatten()
                .flatten();

            match secrets {
                Some(secrets) => secrets.db_passphrase,
                None => {
                    // Invalid metadata & session state. Clean up.
                    std::fs::remove_dir_all(&db_path).unwrap();
                    Alphanumeric.sample_string(&mut rand::rng(), USER_DB_PASSWORD_LEN)
                }
            }
        } else {
            Alphanumeric.sample_string(&mut rand::rng(), USER_DB_PASSWORD_LEN)
        };

        let client = Client::builder()
            .homeserver_url(homeserver.clone())
            .sqlite_store(db_path.clone(), Some(&db_passphrase))
            .handle_refresh_tokens()
            .build()
            .await
            .map_err(|e| AuthError::ConnectionError(e.to_string()))?;

        client
            .matrix_auth()
            .login_username(&username, &password)
            .send()
            .await
            .map_err(|e| AuthError::APIError(e.to_string()))?;

        Ok(client)
    }
}

impl Matrix {
    async fn sync(client: Client) {
        client.add_event_handler(|e: SyncRoomMessageEvent| async move {
            tracing::info!("{:?}", e.as_original().unwrap());
        });

        tracing::info!("Starting sync");

        client.sync(SyncSettings::default()).await.unwrap();
    }
}

fn user_db_dir_path() -> PathBuf {
    let dir = environment::data_dir().join(USER_DB_DIR_NAME);

    if !dir.exists() {
        std::fs::create_dir_all(dir.as_path()).expect("expected permissions to create dir");
    }

    dir
}

fn db_path_for_user(username: &str) -> PathBuf {
    user_db_dir_path().join(username)
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("No existing session found.")]
    NoSession,
    #[error("Invalid session: {0}")]
    InvalidSession(String),
    #[error("No existing keyring entry found for user.")]
    NoKeyring,
    #[error("Failed establish a connection to the matrix servers: {0}")]
    ConnectionError(String),
    #[error("Failed an API request: {0}")]
    APIError(String),
    #[error("Invalid user id: {0}")]
    InvalidUserId(String),
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error during a matrix function call.")]
    MatrixError(#[from] matrix_sdk::Error),
    #[error("Error occured during handling of a previous matrix session.")]
    SessionError(#[from] session::Error),
    #[error("Error connecting client.")]
    ClientError(#[from] matrix_sdk::ClientBuildError),
    #[error("Error during matrix login.")]
    LoginError(String),
}
