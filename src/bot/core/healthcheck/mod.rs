use crate::bot::core::healthcheck::task::{HealthcheckCollectedResult, HealthcheckTask, HealthcheckTaskResult};

pub(crate) mod webhook_info;
pub(crate) mod endpoint;
pub(crate) mod bot_identity;
pub mod task;
pub mod tasks;

pub async fn run_healthcheck() -> HealthcheckCollectedResult {
    let webhook = tasks::webhook::WebhookCheckTask::new();

    let mut tasks = vec![Box::new(webhook)];
    for task in &mut tasks {
        task.execute().await;
    }
    let mut collected_result = true;
    let mut collected_items = vec![];

    for task in &tasks {
        let check = task.check_result();
        match check {
            HealthcheckTaskResult::Failed => {
                collected_result = false;
            }
            HealthcheckTaskResult::Success => {}
            HealthcheckTaskResult::Unchecked => {}
        }
        collected_items.push(task.get_result());

    }
    HealthcheckCollectedResult {
        healthy: collected_result,
        tasks: collected_items,
    }
}


/*
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
                            Ok(telegram_healthcheck_api().await)
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
*/