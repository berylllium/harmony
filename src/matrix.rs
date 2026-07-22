pub mod session;

use futures_util::{StreamExt, pin_mut};
use gpui::*;
use matrix_sdk::{
    AuthSession, Client,
    authentication::matrix::MatrixSession,
    config::SyncSettings,
    ruma::{DeviceId, OwnedUserId, events::room::message::SyncRoomMessageEvent},
};
use matrix_sdk_ui::{
    room_list_service::{RoomListItem, filters::new_filter_non_left},
    sync_service::SyncService,
};
use rand::distr::{Alphanumeric, SampleString};
use tracing::info;

use crate::{
    matrix::session::{SessionMetadata, SessionSecrets},
    tokio_bridge,
};

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
    pub rooms: Vec<RoomListItem>,
}

impl Matrix {
    pub fn new() -> Self {
        Self {
            connection: ConnectionState::CheckingForSession,
            rooms: Vec::new(),
        }
    }
}

// Auth.
impl Matrix {
    /// Attempt authenticating with the homeserver in a background thread.
    pub fn auth(&mut self, cx: &mut App, auth_info: AuthInfo) -> Task<()> {
        info!("Starting an authentication attempt");

        self.connection = match &auth_info {
            AuthInfo::Session => ConnectionState::CheckingForSession,
            AuthInfo::Password { .. } => ConnectionState::Connecting,
        };

        cx.spawn(async move |cx| {
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

            cx.update_global(|matrix: &mut Matrix, cx| {
                matrix.connection = match client {
                    Ok(client) => {
                        // Start sync on successful connection.
                        Self::start_client_sync(client.clone(), cx);
                        Self::start_room_list_sync(client.clone(), cx);

                        ConnectionState::Connected(client)
                    }
                    Err(e) => match auth_info {
                        AuthInfo::Session => {
                            info!("Error during session authentication, skipping auth: {}", e);
                            ConnectionState::AwaitingLogin
                        }
                        AuthInfo::Password { .. } => ConnectionState::Error(e),
                    },
                };
            })
            .unwrap();
        })
    }

    /// Make a new Client connection.
    async fn new_connection(
        homeserver: String,
        user_id: String,
    ) -> Result<(Client, String), AuthError> {
        let device_id = DeviceId::new().to_string();

        let db_path = session::db_dir(&device_id);
        let db_password = Alphanumeric.sample_string(&mut rand::rng(), USER_DB_PASSWORD_LEN);

        let client = Client::builder()
            .homeserver_url(&homeserver)
            .sqlite_store(&db_path, Some(&db_password))
            .handle_refresh_tokens()
            .build()
            .await
            .map_err(|e| AuthError::ConnectionError(e.to_string()))?;

        let meta = SessionMetadata {
            homeserver,
            user_id,
            device_id: device_id.clone(),
        };

        let secrets = SessionSecrets {
            access_token: None,
            refersh_token: None,
            db_password,
        };

        meta.store().await.unwrap();
        secrets.store(&meta.device_id).unwrap();

        Ok((client, device_id))
    }

    /// Restore connection to session.
    async fn restore_connection(device_id: &str) -> Result<Client, AuthError> {
        let Ok(Some(metadata)) = SessionMetadata::load(device_id).await else {
            return Err(AuthError::NoSession);
        };

        let Ok(Some(secrets)) = SessionSecrets::load(device_id) else {
            return Err(AuthError::NoSession);
        };

        let client = Client::builder()
            .homeserver_url(&metadata.homeserver)
            .sqlite_store(session::db_dir(device_id), Some(&secrets.db_password))
            .handle_refresh_tokens()
            .build()
            .await
            .map_err(|e| AuthError::ConnectionError(e.to_string()))?;

        Ok(client)
    }

    async fn auth_session() -> Result<Client, AuthError> {
        info!("Attempting to restore a previous session");

        let Some(device_id) = session::load_last_device_id() else {
            return Err(AuthError::NoSession);
        };

        let (access_token, refresh_token) = {
            let Ok(Some(secrets)) = SessionSecrets::load(&device_id) else {
                return Err(AuthError::NoSession);
            };

            let Some(access_token) = secrets.access_token.clone() else {
                return Err(AuthError::NoSession);
            };

            (access_token, secrets.refersh_token.clone())
        };

        let Ok(Some(metadata)) = SessionMetadata::load(&device_id).await else {
            return Err(AuthError::NoSession);
        };

        let client = Self::restore_connection(&device_id).await?;

        client
            .restore_session(MatrixSession {
                meta: matrix_sdk::SessionMeta {
                    user_id: metadata
                        .user_id
                        .parse::<OwnedUserId>()
                        .map_err(|err| AuthError::InvalidUserId(err.to_string()))?,
                    device_id: device_id.into(),
                },
                tokens: matrix_sdk::SessionTokens {
                    access_token,
                    refresh_token,
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
        info!("Attempting password authentication: {homeserver}, {username}");

        let (client, device_id) = match session::load_last_device_id() {
            Some(last_device_id) => (
                Self::restore_connection(&last_device_id).await?,
                last_device_id,
            ),
            None => Self::new_connection(homeserver.clone(), username.clone()).await?,
        };

        let mut secrets = SessionSecrets::load(&device_id).unwrap().unwrap();

        client
            .matrix_auth()
            .login_username(&username, &password)
            .device_id(&device_id)
            .send()
            .await
            .map_err(|e| AuthError::APIError(e.to_string()))?;

        let session = client
            .session()
            .and_then(|s| match s {
                AuthSession::Matrix(m) => Some(m),
                _ => None,
            })
            .expect("client currently only accepts matrix_auth");

        secrets.access_token = Some(session.tokens.access_token);
        secrets.refersh_token = session.tokens.refresh_token;
        secrets.store(&device_id).unwrap();

        Ok(client)
    }
}

// Syncing.
impl Matrix {
    fn start_client_sync(client: Client, cx: &mut App) {
        tokio_bridge::spawn(cx, async move {
            client.add_event_handler(|e: SyncRoomMessageEvent| async move {
                info!("{:?}", e.as_original().unwrap());
            });

            info!("Starting sync");

            client.sync(SyncSettings::default()).await.unwrap();
        })
        .detach();
    }

    fn start_room_list_sync(client: Client, cx: &mut App) {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Vec<RoomListItem>>();

        tokio_bridge::spawn(cx, async move {
            let sync_service = match SyncService::builder(client).build().await {
                Ok(sync_service) => sync_service,
                Err(err) => {
                    tracing::error!("Failed to build matrix sync service: {:?}", err);
                    return;
                }
            };

            let room_list_service = sync_service.room_list_service();
            let all_rooms = match room_list_service.all_rooms().await {
                Ok(all_rooms) => all_rooms,
                Err(err) => {
                    tracing::error!("Failed to get all_rooms from room list service: {:?}", err);
                    return;
                }
            };

            let (entries_stream, controller) = all_rooms.entries_with_dynamic_adapters(50);
            controller.set_filter(Box::new(new_filter_non_left()));
            pin_mut!(entries_stream);

            sync_service.start().await;

            let mut rooms = eyeball_im::Vector::<RoomListItem>::new();
            while let Some(diffs) = entries_stream.next().await {
                for diff in diffs {
                    diff.apply(&mut rooms);
                }

                if tx.send(rooms.iter().cloned().collect()).is_err() {
                    break;
                }
            }
        })
        .detach();

        cx.spawn(async move |cx| {
            while let Some(rooms) = rx.recv().await {
                if cx
                    .update_global(|s: &mut Matrix, _| {
                        s.rooms = rooms;
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
    }
}

impl Global for Matrix {}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("No existing session found.")]
    NoSession,
    #[error("Invalid session: {0}")]
    InvalidSession(String),
    #[error("Failed establish a connection to the matrix servers: {0}")]
    ConnectionError(String),
    #[error("Failed an API request: {0}")]
    APIError(String),
    #[error("Invalid user id: {0}")]
    InvalidUserId(String),
}
