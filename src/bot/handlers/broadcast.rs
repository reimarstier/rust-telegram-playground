use teloxide::Bot;
use teloxide::prelude::{Message, Requester};
use teloxide::types::ChatId;
use tracing::debug;

use crate::bot::{HandlerResult, MyDialogue, State};
use crate::bot::core::db::client::DatabaseClient;

pub(crate) async fn broadcast_start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Send me the text to broadcast a message to all users.").await?;
    dialogue.update(State::Broadcast).await?;
    Ok(())
}

pub(crate) async fn receive_broadcast_message(bot: Bot, dialogue: MyDialogue, msg: Message, db_client: DatabaseClient) -> HandlerResult {
    match msg.text().map(ToOwned::to_owned) {
        Some(broadcast_message) => {
            let reply = format!("Sending broadcast to all users:\n{}", broadcast_message);
            bot.send_message(msg.chat.id, reply).await?;

            let users = db_client.list_registered_users().await?;
            for user in users {
                let id = ChatId(user.telegram_id.unwrap());
                debug!("Sending broadcast to {}", id);
                bot.send_message(id, broadcast_message.clone()).await?;
            }
            dialogue.update(State::Start).await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please, send me a proper broadcast message.").await?;
        }
    }

    Ok(())
}