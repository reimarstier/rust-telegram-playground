use anyhow::anyhow;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::bot::core::bot_config::BotConfig;
use crate::bot::core::bot_config::webhook::BotConfigWebHook;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub(crate) struct BotHealthResult {
    ok: bool,
}

pub(crate) async fn healthcheck() -> Json<BotHealthResult> {
    Json(BotHealthResult { ok: true })
}

/// See https://core.telegram.org/bots/api#getwebhookinfo
#[derive(Deserialize, Debug)]
pub(crate) struct TelegramHealthResult {
    /// Webhook URL, may be empty if webhook is not set up
    #[serde(rename = "url")]
    url: String,
    /// True, if a custom certificate was provided for webhook certificate checks
    #[serde(rename = "has_custom_certificate")]
    _has_custom_certificate: bool,
    /// Number of updates awaiting delivery
    #[serde(rename = "pending_update_count")]
    _pending_update_count: u64,
    /// Optional. Currently used webhook IP address
    #[serde(rename = "ip_address")]
    _ip_address: Option<String>,
    /// Optional. Error message in human-readable format for the most recent error that happened when trying to deliver an update via webhook
    #[serde(rename = "last_error_message")]
    _last_error_message: Option<String>,
    /// Optional. Unix time for the most recent error that happened when trying to deliver an update via webhook
    #[serde(rename = "last_error_date")]
    _last_error_date: Option<String>,
    /// The maximum allowed number of simultaneous HTTPS connections to the webhook for update delivery, 1-100. Defaults to 40.
    /// Use lower values to limit the load on your bot's server, and higher values to increase your bot's throughput.
    #[serde(rename = "max_connections")]
    _max_connections: Option<u64>,

}

#[derive(Deserialize, Debug)]
pub(crate) struct TelegramHealthReply {
    pub ok: Option<bool>,
    /// Error code 404 may indicate unknown/invalid bot authentication credentials.
    pub error_code: Option<u64>,
    pub result: TelegramHealthResult,
}

impl TelegramHealthReply {
    pub(crate) fn healthy(&self) -> TelegramHealthAssessment {
        if self.ok.is_none() || self.error_code.is_some() {
            if let Some(404) = self.error_code {
                TelegramHealthAssessment { healthy: false, message: Some("Invalid/missing authentication.".to_string()) }
            } else {
                TelegramHealthAssessment { healthy: false, message: None }
            }
        } else {
            TelegramHealthAssessment { healthy: true, message: None }
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct TelegramHealthAssessment {
    pub healthy: bool,
    pub message: Option<String>,
}

pub(crate) async fn telegram_healthcheck_api() -> Result<TelegramHealthAssessment, anyhow::Error> {
    let bot_config = BotConfig::new()?;
    let bot_webhook_config = BotConfigWebHook::new()?;

    let webhook_info_url = format!("https://api.telegram.org/bot{}/getWebhookInfo", bot_config.bot_token);
    let body = reqwest::get(webhook_info_url)
        .await?
        .json::<serde_json::Value>()
        //.json::<TelegramHealthReply>()
        .await?;
    println!("Health result telegram api json: {}", body);

    let telegram_reply = serde_json::from_value::<TelegramHealthReply>(body);
    match telegram_reply {
        Ok(reply) => {
            let assessment = reply.healthy();
            println!("Got health reply: {:?}", reply);
            println!("Health assessment: {:?}", assessment);
            if reply.result.url.ne(&bot_webhook_config.public_bot_url.to_string()) {
                Err(anyhow!("Public url in telegram api does not match public url configure in bot! Found api public_url={} configured={:?}",
                    reply.result.url, bot_webhook_config.public_bot_url.to_string()))
            } else {
                Ok(assessment)
            }
        }
        Err(error) => {
            Err(anyhow!("Failed to parse reply. Error: {}", error))
        }
    }
}

pub async fn telegram_healthcheck() -> Result<TelegramHealthAssessment, anyhow::Error> {
    let bot_config = BotConfigWebHook::new()?;
    let bot_health_reply = reqwest::get(bot_config.public_healthcheck_url)
        .await;
    match bot_health_reply {
        Ok(response) => {
            let status_code = response.status();
            let response_result = response.text().await;

            match response_result {
                Ok(response_string) => {
                    match serde_json::from_str::<BotHealthResult>(response_string.clone().as_ref()) {
                        Ok(reply_json) => {
                            println!("Health result json of own bot: {:?}", reply_json);
                            telegram_healthcheck_api().await
                        }
                        Err(error) => {
                            println!("Response of bot is: {:?}", response_string);
                            println!("Status code of bot response is: {:?}", status_code);
                            Err(anyhow!("Could not parse response of bot. Got error: {}", error))
                        }
                    }
                }
                Err(error) => {
                    Err(anyhow!("Could not fetch response of bot. Got error: {}", error))
                }
            }
        }
        Err(error) => {
            Err(anyhow!("Bot did not respond with ok. Got error: {}", error))
        }
    }
}
