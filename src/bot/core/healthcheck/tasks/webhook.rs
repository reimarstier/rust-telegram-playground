use crate::bot::core::bot_config::webhook::BotConfigWebHook;
use crate::bot::core::healthcheck::task::{HealthcheckResult, HealthcheckTask, HealthcheckTaskResult, TaskResult};
use crate::bot::core::healthcheck::webhook_info;
use crate::bot::core::healthcheck::webhook_info::{TelegramUpdateMode, WebhookCheckResult};

pub struct WebhookCheckTask {
    result: Option<WebhookCheckResult>,
}

impl WebhookCheckTask {
    pub fn new() -> Self {
        Self {
            result: None,
        }
    }
    async fn telegram_healthcheck_api() -> WebhookCheckResult {
        let webhook_response = webhook_info::telegram_check_webhook_info().await;
        let bot_webhook_config = BotConfigWebHook::new();

        match (webhook_response, bot_webhook_config) {
            (Ok(webhook_response), Ok(webhook_config)) => {
                // webhook configured
                webhook_response.check_webhook_mode(webhook_config.public_bot_url.as_ref())
            }
            (Ok(webhook_response), Err(_webhook_error)) => {
                // either using getUpdates or missing environment variables for webhook
                webhook_response.check_get_updates_mode()
            }
            (Err(webhook_error), _) => {
                WebhookCheckResult {
                    healthy: false,
                    mode: TelegramUpdateMode::Error,
                    message: Option::from(format!("WebhookInfoError: {}", webhook_error)),
                    result: None,
                }
            }
        }
    }
}

impl HealthcheckTask for WebhookCheckTask {
    async fn execute(&mut self) {
        self.result = Some(WebhookCheckTask::telegram_healthcheck_api().await);
    }

    fn check_result(&self) -> HealthcheckTaskResult {
        match self.result.clone() {
            None => {
                HealthcheckTaskResult::Unchecked
            }
            Some(result) => {
                if result.healthy {
                    HealthcheckTaskResult::Success
                } else {
                    HealthcheckTaskResult::Failed
                }
            }
        }
    }

    fn get_result(&self) -> TaskResult {
        match self.result.clone() {
            None => {
                TaskResult {
                    name: "WebhookInfo".to_string(),
                    result: HealthcheckResult::NoResult("No result".to_string()),
                }
            }
            Some(result) => {
                TaskResult {
                    name: "WebhookInfo".to_string(),
                    result: HealthcheckResult::WebhookInfo(result),
                }
            }
        }

    }
}
