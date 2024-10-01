use crate::bot::core::bot_config::webhook::BotConfigWebHook;
use crate::bot::core::healthcheck::task::{HealthcheckTask, HealthcheckTaskResult, TaskResult};

pub struct HealthEndpointCheck {

}

impl HealthcheckTask for HealthEndpointCheck {
    async fn execute(&mut self) {
        let bot_config = BotConfigWebHook::new();
        match bot_config {
            Ok(bot_config) => {
                let _bot_health_reply = reqwest::get(bot_config.public_healthcheck_url)
                    .await;
                todo!()
            }
            Err(_error) => {
                // webhook configuration error

            }
        }
    }

    fn check_result(&self) -> HealthcheckTaskResult {
        todo!()
    }

    fn get_result(&self) -> TaskResult {
        todo!()
    }
}