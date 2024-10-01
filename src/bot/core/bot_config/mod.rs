use std::env;

use serde::Deserialize;

use crate::bot::core::bot_config::storage::BotStorageConfig;

pub(crate) mod storage;
pub(crate) mod webhook;

const TELOXIDE_TOKEN_KEY: &str = "TELOXIDE_TOKEN";
const TELOXIDE_LOG_DIR_KEY: &str = "TELOXIDE_LOG_DIR";
const TELOXIDE_DATA_DIR_KEY: &str = "TELOXIDE_DATA_DIR";
const TELOXIDE_BIND_PORT_KEY: &str = "TELOXIDE_BIND_PORT";
const TELOXIDE_BIND_ADDRESS_KEY: &str = "TELOXIDE_BIND_ADDRESS";
const TELOXIDE_PUBLIC_URL_KEY: &str = "TELOXIDE_PUBLIC_URL";
pub const TELOXIDE_BOT_NAME_KEY: &str = "TELOXIDE_BOT_NAME";
pub const TELEGRAM_BOT_ENDPOINT_BOT: &str = "/bot";
pub const TELEGRAM_BOT_ENDPOINT_HEALTHCHECK: &str = "/healthcheck";
const DATABASE_FILE_NAME: &str = "db.sqlite";

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct BotConfig {
    pub bot_token: String,
    pub storage: BotStorageConfig,
}

impl BotConfig {
    pub fn new() -> Result<Self, anyhow::Error> {
        let bot_token = env::var(TELOXIDE_TOKEN_KEY).expect("Could not find telegram bot token. Check env var TELOXIDE_TOKEN.");
        let bot_storage_config = BotStorageConfig::new()?;

        Ok(Self {
            bot_token,
            storage: bot_storage_config,
        })
    }
}
