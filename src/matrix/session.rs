use std::path::PathBuf;

use crate::environment;

const KEYRING_SERVICE: &'static str = "matrix-harmony";

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SessionMetadata {
    pub homeserver: String,
    pub user_id: String,
    pub device_id: String,
}

impl SessionMetadata {
    pub async fn store(&self) -> Result<(), Error> {
        tokio::fs::write(Self::path(&self.device_id), serde_json::to_string(self)?)
            .await
            .expect("expected file writing permissions");

        Ok(())
    }

    pub async fn load(device_id: &str) -> Result<Option<Self>, Error> {
        match tokio::fs::read_to_string(Self::path(device_id)).await {
            Ok(json) => Ok(Some(serde_json::from_str(&json)?)),
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => Ok(None),
                _ => panic!("expected file reading permissions"),
            },
        }
    }

    fn path(device_id: &str) -> PathBuf {
        session_dir(device_id).join("metadata.json")
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SessionSecrets {
    pub access_token: Option<String>,
    pub refersh_token: Option<String>,
    pub db_password: String,
}

impl SessionSecrets {
    pub fn store(&self, device_id: &str) -> Result<(), Error> {
        let entry = keyring::Entry::new(KEYRING_SERVICE, device_id)?;
        let json = serde_json::to_string(self)?;
        entry.set_password(&json)?;

        Ok(())
    }

    pub fn load(device_id: &str) -> Result<Option<SessionSecrets>, Error> {
        let entry = keyring::Entry::new(KEYRING_SERVICE, device_id)?;

        match entry.get_password() {
            Ok(json) => Ok(Some(serde_json::from_str(&json)?)),
            Err(keyring::Error::NoEntry) => Ok(None), // No session stored, not an error.
            Err(e) => Err(e.into()),
        }
    }

    /// Deletes the stored secrets for the given user from the OS keyring.
    pub fn delete(user_id: &str) -> Result<(), Error> {
        let entry = keyring::Entry::new(KEYRING_SERVICE, user_id)?;
        entry.delete_credential()?;
        Ok(())
    }
}

pub fn exists(device_id: &str) -> bool {
    session_dir(device_id).exists()
}

pub fn delete(device_id: &str) {
    std::fs::remove_dir_all(session_dir(device_id)).unwrap()
}

pub fn store_last_device_id(device_id: &str) {
    std::fs::write(last_device_id_path(), device_id).unwrap();
}

pub fn load_last_device_id() -> Option<String> {
    std::fs::read_to_string(last_device_id_path()).ok()
}

pub fn last_device_id_exists() -> bool {
    last_device_id_path().exists()
}

pub fn db_dir(device_id: &str) -> PathBuf {
    let dir = session_dir(device_id).join("db");

    if !dir.exists() {
        std::fs::create_dir_all(dir.as_path()).expect("expected permissions to create dir");
    }

    dir
}

fn session_dir(device_id: &str) -> PathBuf {
    environment::data_dir().join(device_id)
}

fn sessions_dir() -> PathBuf {
    let dir = environment::data_dir().join("sessions");

    if !dir.exists() {
        std::fs::create_dir_all(dir.as_path()).expect("expected permissions to create dir");
    }

    dir
}

fn last_device_id_path() -> PathBuf {
    cache_dir().join("last_device_id")
}

fn cache_dir() -> PathBuf {
    let dir = environment::cache_dir();

    if !dir.exists() {
        std::fs::create_dir_all(dir.as_path()).expect("expected permissions to create dir");
    }

    dir
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error during system keyring operation.")]
    Keyring(#[from] keyring::Error),
    #[error("Error during json parsing.")]
    ParseJson(#[from] serde_json::Error),
}
