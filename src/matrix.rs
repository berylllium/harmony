pub mod session;

use std::{assert_matches, path::PathBuf};

use gpui::*;
use matrix_sdk::{AuthSession, Client, authentication::matrix::MatrixSession, ruma::OwnedUserId};
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
    Error(Error),
}

pub struct Matrix {
    pub connection: ConnectionState,
}

impl Matrix {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mut s = Self {
            connection: ConnectionState::CheckingForSession,
        };

        s.try_restore(cx);
        s
    }

    pub fn entity(cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(cx))
    }

    pub fn try_restore(&mut self, cx: &mut Context<Self>) {
        assert_matches!(self.connection, ConnectionState::CheckingForSession);

        cx.spawn(async move |s, cx| {
            let result: Result<Option<Client>, Error> = tokio_bridge::spawn_async(cx, async move {
                let Some(metadata) = SessionMetadata::load().await.unwrap() else {
                    return Ok(None); // No saved metadata, not an error.
                };

                let user_id = metadata.user_id.clone();
                let secrets = SessionSecrets::load(&user_id)?;

                let Some(secrets) = secrets else {
                    return Ok(None); // Metadata exists, no keychain entry: treat as logged out.
                };

                let client = Client::builder()
                    .homeserver_url(metadata.homeserver)
                    .sqlite_store(metadata.db_path, Some(&secrets.db_passphrase))
                    .handle_refresh_tokens()
                    .build()
                    .await?;

                client
                    .restore_session(MatrixSession {
                        meta: matrix_sdk::SessionMeta {
                            user_id: metadata
                                .user_id
                                .parse::<OwnedUserId>()
                                .map_err(|err| Error::LoginError(err.to_string()))?,
                            device_id: secrets.device_id.into(),
                        },
                        tokens: matrix_sdk::SessionTokens {
                            access_token: secrets.access_token,
                            refresh_token: secrets.refersh_token,
                        },
                    })
                    .await?;

                Ok(Some(client))
            })
            .await
            .unwrap();

            s.update(cx, |s, cx| {
                s.connection = match result {
                    Ok(Some(client)) => ConnectionState::Connected(client),
                    Ok(None) => ConnectionState::AwaitingLogin,
                    Err(e) => ConnectionState::Error(e),
                };

                cx.notify();
            })
        })
        .detach();
    }

    pub fn login_password(
        &mut self,
        homeserver: String,
        username: String,
        password: String,
        cx: &mut Context<Self>,
    ) {
        self.connection = ConnectionState::Connecting;
        cx.notify();

        cx.spawn(async move |s, cx| {
            let result: Result<Client, Error> = tokio_bridge::spawn_async(cx, async move {
                let db_path = db_path_for_user(&username);
                let db_passphrase = if db_path.exists() {
                    let secrets = SessionMetadata::load()
                        .await
                        .unwrap()
                        .map(|m| SessionSecrets::load(&m.user_id))
                        .transpose()
                        .unwrap()
                        .flatten();

                    match secrets {
                        Some(secrets) => secrets.db_passphrase,
                        None => {
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
                    .await?;

                client
                    .matrix_auth()
                    .login_username(&username, &password)
                    .send()
                    .await?;

                let session = client
                    .session()
                    .and_then(|s| match s {
                        AuthSession::Matrix(m) => Some(m),
                        _ => None,
                    })
                    .expect("client currently only accepts matrix_auth");

                let metadata = SessionMetadata {
                    homeserver: homeserver,
                    db_path,
                    user_id: session.meta.user_id.to_string(),
                };

                metadata.store().await.unwrap();

                let secrets = SessionSecrets {
                    access_token: session.tokens.access_token,
                    refersh_token: session.tokens.refresh_token,
                    device_id: session.meta.device_id.to_string(),
                    db_passphrase,
                };

                secrets.store(&metadata.user_id).unwrap();

                Ok(client)
            })
            .await
            .unwrap();

            s.update(cx, |s, cx| {
                s.connection = match result {
                    Ok(client) => ConnectionState::Connected(client),
                    Err(e) => {
                        tracing::error!("Error during matrix connection: {:?}", e);
                        ConnectionState::Error(e)
                    }
                };
                cx.notify();
            })
            .unwrap()
        })
        .detach();
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
