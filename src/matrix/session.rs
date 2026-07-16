use std::path::PathBuf;

use crate::environment;

const KEYRING_SERVICE: &'static str = "matrix-harmony";

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SessionMetadata {
    pub homeserver: String,
    pub db_path: PathBuf,
    pub user_id: String,
}

impl SessionMetadata {
    pub async fn store(&self) -> Result<(), Error> {
        tokio::fs::write(metadata_path(), serde_json::to_string(self)?)
            .await
            .expect("expected file writing permissions");

        Ok(())
    }

    pub async fn load() -> Result<Option<Self>, Error> {
        match tokio::fs::read_to_string(metadata_path()).await {
            Ok(json) => Ok(Some(serde_json::from_str(&json)?)),
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => Ok(None),
                _ => panic!("expected file reading permissions"),
            },
        }
    }

    pub fn exists() -> bool {
        metadata_path().exists()
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SessionSecrets {
    pub access_token: String,
    pub refersh_token: Option<String>,
    pub device_id: String,
    pub db_passphrase: String,
}

impl SessionSecrets {
    pub fn store(&self, user_id: &str) -> Result<(), Error> {
        let entry = keyring::Entry::new(KEYRING_SERVICE, user_id)?;
        let json = serde_json::to_string(self)?;
        entry.set_password(&json)?;

        Ok(())
    }

    pub fn load(user_id: &str) -> Result<Option<SessionSecrets>, Error> {
        let entry = keyring::Entry::new(KEYRING_SERVICE, user_id)?;

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

fn metadata_path() -> PathBuf {
    environment::data_dir().join("user_metadata.json")
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error during system keyring operation.")]
    Keyring(#[from] keyring::Error),
    #[error("Error during json parsing.")]
    ParseJson(#[from] serde_json::Error),
}
