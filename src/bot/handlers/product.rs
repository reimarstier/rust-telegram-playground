use teloxide::Bot;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::{CallbackQuery, Message, Requester};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use crate::bot::{HandlerResult, MyDialogue, State};

pub(crate) async fn start_purchase(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Let's start! What's your full name?").await?;
    dialogue.update(State::PurchaseReceiveFullName).await?;
    Ok(())
}

pub(crate) async fn receive_full_name(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text().map(ToOwned::to_owned) {
        Some(full_name) => {
            let products = ["Apple", "Banana", "Orange", "Potato"]
                .map(|product| InlineKeyboardButton::callback(product, product));

            bot.send_message(msg.chat.id, "Select a product:")
                .reply_markup(InlineKeyboardMarkup::new([products]))
                .await?;
            dialogue.update(State::ReceiveProductChoice { full_name }).await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please, send me your full name.").await?;
        }
    }

    Ok(())
}

pub(crate) async fn receive_product_selection(
    bot: Bot,
    dialogue: MyDialogue,
    full_name: String, // Available from `State::ReceiveProductChoice`.
    q: CallbackQuery,
) -> HandlerResult {
    if let Some(product) = &q.data {
        bot.send_message(
            dialogue.chat_id(),
            format!("{full_name}, product '{product}' has been purchased successfully!"),
        )
            .await?;
        dialogue.exit().await?;
    }

    Ok(())
}
