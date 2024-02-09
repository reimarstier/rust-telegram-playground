use std::env;
use anyhow::anyhow;
use std::net::SocketAddr;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use crate::bot::core::bot_config::{TELOXIDE_BIND_ADDRESS_KEY, TELOXIDE_BIND_PORT_KEY, TELOXIDE_PUBLIC_URL_KEY};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub(crate) struct BotConfigWebHook {
    pub socket_address: SocketAddr,
    pub public_url: Url,
    pub public_bot_url: Url,
    pub public_healthcheck_url: Url,
}

impl BotConfigWebHook {
    pub fn new() -> Result<Self, anyhow::Error> {
        let bind_port = env::var(TELOXIDE_BIND_PORT_KEY)
            .map_err(|_| anyhow!("Could not find environment variable {}", TELOXIDE_BIND_PORT_KEY))?;
        let bind_port = bind_port.parse::<u16>().map_err(|error|
            anyhow!("Could not parse bind port. Check environment variable '{}={}'. Error: {}", TELOXIDE_BIND_PORT_KEY, bind_port, error)
        )?;

        let bind_address = env::var(TELOXIDE_BIND_ADDRESS_KEY)
            .unwrap_or("0.0.0.0".to_string());
        let socket_address: SocketAddr = format!("{}:{}", bind_address, bind_port).parse()
            .map_err(|error| {
                anyhow!("Unable to parse socket address. Check environment vars: {}. Error: {}", TELOXIDE_BIND_ADDRESS_KEY, error)
            })?;

        let public_url_string = env::var(TELOXIDE_PUBLIC_URL_KEY)
            .map_err(|_error| anyhow!("Could not get public bot address. Ensure environment variable '{}' exists.", TELOXIDE_PUBLIC_URL_KEY))?;
        let public_url: Url = public_url_string.parse()
            .map_err(|error| anyhow!("Failed to parse public url in environment variable '{}'. Error: {}", TELOXIDE_PUBLIC_URL_KEY, error))?;

        let public_bot_url: Url = public_url.join("/bot")?;
        let public_healthcheck_url: Url = public_url.join("/healthcheck")?;

        Ok(Self {
            socket_address,
            public_url,
            public_bot_url,
            public_healthcheck_url,
        })
    }
}
