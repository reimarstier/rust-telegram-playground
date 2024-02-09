use std::{env, fs};
use std::path::PathBuf;

use anyhow::anyhow;
use serde::Deserialize;

use crate::bot::core::bot_config::{DATABASE_FILE_NAME, TELOXIDE_DATA_DIR_KEY, TELOXIDE_LOG_DIR_KEY};

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct BotStorageConfig {
    pub log_directory: String,
    pub data_directory: String,
}

impl BotStorageConfig {
    pub fn new() -> Result<Self, anyhow::Error> {
        let log_directory = env::var(TELOXIDE_LOG_DIR_KEY).unwrap_or("/var/log/telegrambot/".to_string());
        let data_directory = env::var(TELOXIDE_DATA_DIR_KEY).unwrap_or("/var/lib/telegrambot/".to_string());
        Self::check_directory_write_access(&log_directory, TELOXIDE_LOG_DIR_KEY)?;
        Self::check_directory_write_access(&log_directory, TELOXIDE_DATA_DIR_KEY)?;

        Ok(Self {
            log_directory,
            data_directory,
        })

    }
    pub fn database_url(&self) -> Result<String, anyhow::Error> {
        let string = self.database_path().into_os_string().into_string()
            .map_err(|e| anyhow!("Failed to convert database path to string: {:?}", e))?;
        Ok(format!("sqlite://{}", string))
    }

    pub fn database_path(&self) -> PathBuf {
        PathBuf::from(self.data_directory.clone()).join(DATABASE_FILE_NAME)
    }

    fn check_directory_write_access(directory: &str, source_env_key: &str) -> Result<bool, anyhow::Error> {
        let metadata = fs::metadata(directory)?;
        if !metadata.is_dir() {
            return Err(anyhow!("Expected a directory in environment variable {}!", source_env_key));
        }
        let directory_owner_id = {
            use std::os::linux::fs::MetadataExt;
            metadata.st_uid()
        };
        let process_owner_id = Self::process_owner_id()?;
        if directory_owner_id.ne(&process_owner_id) {
            return Err(anyhow!("Expected write access to directory in environment variable {}!", source_env_key));
        }

        Ok(true)
    }

    fn process_owner_id() -> Result<u32, anyhow::Error> {
        use std::os::unix::fs::MetadataExt;
        let user_id = std::fs::metadata("/proc/self").map(|m| m.uid())?;
        Ok(user_id)
    }
}