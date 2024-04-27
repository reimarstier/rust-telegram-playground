use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use crate::bot::core::bot_config::BotConfig;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum TelegramUpdateMode {
    GetUpdates,
    Webhook,
    Error,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub(crate) struct WebhookCheckResult {
    pub healthy: bool,
    pub mode: TelegramUpdateMode,
    pub message: Option<String>,
    pub result: Option<TelegramWebhookInfoResult>,
}

/// See https://core.telegram.org/bots/api#getwebhookinfo
#[derive(Deserialize, Serialize, Clone, Debug)]
pub(crate) struct TelegramWebhookInfoResult {
    /// Webhook URL, may be empty if webhook is not set up
    #[serde(rename = "url")]
    pub url: String,
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

#[derive(Deserialize, Serialize, Clone, Debug)]
pub(crate) struct TelegramWebhookInfoResponse {
    pub ok: Option<bool>,
    /// Error code 404 may indicate unknown/invalid bot authentication credentials.
    pub error_code: Option<u64>,
    pub result: TelegramWebhookInfoResult,
}

impl TelegramWebhookInfoResponse {
    fn healthy(&self, mode: TelegramUpdateMode) -> WebhookCheckResult {
        if self.ok.is_none() || self.error_code.is_some() {
            if let Some(404) = self.error_code {
                WebhookCheckResult {
                    healthy: false,
                    mode,
                    message: Some("Invalid/missing authentication.".to_string()),
                    result: Some(self.result.clone()),
                }
            } else {
                WebhookCheckResult {
                    healthy: false,
                    mode,
                    message: Some("Unknown error".to_string()),
                    result: Some(self.result.clone()),
                }
            }
        } else {
            WebhookCheckResult {
                healthy: true,
                mode,
                message: None,
                result: Some(self.result.clone()),
            }
        }
    }

    pub(crate) fn check_get_updates_mode(&self) -> WebhookCheckResult {
        self.healthy(TelegramUpdateMode::GetUpdates)
    }

    pub(crate) fn check_webhook_mode(&self, public_bot_url: &str) -> WebhookCheckResult {
        let assessment = self.healthy(TelegramUpdateMode::Webhook);
        if self.result.url.ne(public_bot_url) {
            WebhookCheckResult {
                healthy: false,
                mode: TelegramUpdateMode::Webhook,
                message: Some(format!("Public url in telegram api does not match public url configure in bot! Found api public_url={} configured={:?}",
                                      self.result.url.clone(), public_bot_url)),
                result: Some(self.result.clone()),
            }
        } else {
            assessment
        }
    }
}

pub(crate) async fn telegram_check_webhook_info() -> Result<TelegramWebhookInfoResponse, anyhow::Error> {
    let bot_config = BotConfig::new()?;

    let webhook_info_url = format!("https://api.telegram.org/bot{}/getWebhookInfo", bot_config.bot_token);
    let body = reqwest::get(webhook_info_url)
        .await?
        .json::<serde_json::Value>()
        .await?;

    serde_json::from_value::<TelegramWebhookInfoResponse>(body.clone())
        .map_err(|error| anyhow!("Could not parse webhook info: {}. Error: {}", body, error))
}
