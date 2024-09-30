use teloxide::{Bot, dptree};
use teloxide::dispatching::{dialogue, HandlerExt, UpdateFilterExt, UpdateHandler};
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::{Message, Requester, Update};
use teloxide::utils::command::BotCommands;

use crate::bot::{HandlerResult, MyDialogue, State};
use crate::bot::core::db::client::DatabaseClient;
use crate::bot::handlers::{product, broadcast, search};
use crate::bot::handlers::register::register;

/// These commands are supported:
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub(crate) enum BasicCommands {
    #[command(description = "Display this text")]
    Help,
    #[command(description = "Register with this bot")]
    Start(String),
    #[command(description = "Cancel a dialogue")]
    Cancel,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum UserCommands {
    #[command(description = "Purchase product")]
    Purchase,
    #[command(description = "Search for aliases")]
    Search,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum AdminCommands {
    #[command(description = "Send a message to all registered users.")]
    Broadcast,
}

pub(crate) fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;

    let basic_command_handler = teloxide::filter_command::<BasicCommands, _>()
        .branch(
            case![State::Start]
                .branch(case![BasicCommands::Help].endpoint(help))
                .branch(case![BasicCommands::Start(token)].endpoint(register))
        )
        .branch(case![BasicCommands::Cancel].endpoint(cancel));

    let user_command_handler = Update::filter_message()
        .branch(
            case![State::Start]
                .branch(
                    dptree::filter(|database_client: DatabaseClient, msg: Message| {
                    msg.from.map(|user| database_client.known_user_exists(user.id.0 as i64)).unwrap_or(false)
                })
                .filter_command::<UserCommands>()
                .branch(case![UserCommands::Search].endpoint(search::search_start))
                .branch(case![UserCommands::Purchase].endpoint(product::start_purchase)),
            )
        );

    let admin_command_handler = Update::filter_message()
        .branch(
            case![State::Start]
                .branch(
                    dptree::filter(|database_client: DatabaseClient, msg: Message| {
                        msg.from.map(|user| database_client.known_admin_user_exists(user.id.0 as i64)).unwrap_or(false)
                    })
                        .filter_command::<AdminCommands>()
                        .branch(case![AdminCommands::Broadcast].endpoint(broadcast::broadcast_start)),
                )
        );

    let primary_stage_handlers = Update::filter_message()
        .branch(basic_command_handler)
        .branch(user_command_handler)
        .branch(admin_command_handler);

    let second_stage_handlers = Update::filter_message()
        .branch(case![State::Search].endpoint(search::receive_search_query))
        .branch(case![State::Broadcast].endpoint(broadcast::receive_broadcast_message))
        .branch(case![State::PurchaseReceiveFullName].endpoint(product::receive_full_name));

    let message_handler = Update::filter_message()
        .branch(primary_stage_handlers)
        .branch(second_stage_handlers)
        // fallback
        .branch(dptree::endpoint(invalid_state));

    let callback_query_handler = Update::filter_callback_query().branch(
        case![State::ReceiveProductChoice { full_name }].endpoint(product::receive_product_selection),
    );

    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(message_handler)
        .branch(callback_query_handler)
}


async fn help(bot: Bot, msg: Message, database_client: DatabaseClient) -> HandlerResult {
    let basic_commands = format!("Basic commands:\n{}", BasicCommands::descriptions().to_string());
    let user_commands = format!("User commands:\n{}", UserCommands::descriptions().to_string());
    let admin_commands = format!("Admin commands:\n{}", AdminCommands::descriptions().to_string());
    if database_client.known_admin_user_exists(msg.chat.id.0) {
        let response = format!("{}\n\n{}\n\n{}", basic_commands, user_commands, admin_commands);
        bot.send_message(msg.chat.id, response).await?;
    } else if database_client.known_user_exists(msg.chat.id.0) {
        let response = format!("{}\n\n{}", basic_commands, user_commands);
        bot.send_message(msg.chat.id, response).await?;
    } else {
        let response = format!("You are not registered. Please type /start <start_token> to register.\n\n{}", basic_commands);
        bot.send_message(msg.chat.id, response).await?;
    }
    Ok(())
}

async fn cancel(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Cancelling the dialogue.").await?;
    dialogue.exit().await?;
    Ok(())
}

async fn invalid_state(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Unable to handle the message. Type /help to see the usage.")
        .await?;
    Ok(())
}


