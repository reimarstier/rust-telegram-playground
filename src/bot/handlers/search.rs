use teloxide::Bot;
use teloxide::prelude::{Message, Requester};

use crate::bot::{HandlerResult, MyDialogue, State};
use crate::bot::core::db::client::DatabaseClient;

pub(crate) async fn search(bot: Bot, dialogue: MyDialogue, msg: Message, db_client: DatabaseClient) -> HandlerResult {
    bot.send_message(msg.chat.id, "Did not recognize you.").await?;
    tracing::info!("Initiating search for user id: {:?}", db_client.known_user(msg.chat.id.0));
    dialogue.update(State::Search).await?;
    Ok(())
}


pub(crate) async fn receive_search_query(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text().map(ToOwned::to_owned) {
        Some(search_string) => {
            bot.send_message(msg.chat.id, format!("Searching for {}", search_string)).await?;
            dialogue.update(State::Start).await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please, send me a proper search query.").await?;
        }
    }

    Ok(())
}
