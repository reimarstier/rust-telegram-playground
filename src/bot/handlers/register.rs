use teloxide::Bot;
use teloxide::prelude::{Message, Requester};
use teloxide::types::Me;
use teloxide::utils::command::BotCommands;
use crate::bot::core::db::client::admin_client::DatabaseAdminClient;
use crate::bot::core::db::client::DatabaseClient;
use crate::bot::core::db::DatabaseError;
use crate::bot::HandlerResult;

pub(crate) async fn register(bot: Bot, msg: Message, mut database_client: DatabaseClient, me: Me) -> HandlerResult {
    match msg.text().map(|data| crate::bot::schema::BasicCommands::parse(data, me.username())) {
        Some(Ok(crate::bot::schema::BasicCommands::Start(token))) => {
            if token.is_empty() {
                bot.send_message(msg.chat.id, "Did not receive any data from you.").await?;
            } else {
                tracing::debug!("Received start token {} from telegram account id={}", token, msg.chat.id.0);
                let telegram_id = msg.chat.id.0;
                let result = database_client.register_telegram_account_of_user(&token, telegram_id).await;
                match result {
                    Ok(_telegram_account) => {
                        bot.send_message(msg.chat.id, "You were successfully registered.").await?;
                    }
                    Err(error) => {
                        tracing::error!("Error adding user for telegram account id={} start={}: {}", msg.chat.id.0, token, error);
                        match error {
                            DatabaseError::UnknownUser(_error) => {
                                bot.send_message(msg.chat.id, "Could not find the user.").await?;
                            }
                            DatabaseError::CreateError(_error) => {
                                bot.send_message(msg.chat.id, "Could not create the user.").await?;
                            }
                            _ => {
                                bot.send_message(msg.chat.id, "An error occurred.").await?;
                            }
                        }
                    }
                }
            }
        }
        Some(Err(error)) => {
            bot.send_message(msg.chat.id, "Could not parse data.").await?;

            tracing::error!("Error parsing start command: {}", error);
        }
        _ => {
            bot.send_message(msg.chat.id, "Did not receive the expected data from you.").await?;
        }
    }
    Ok(())
}
