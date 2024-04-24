use teloxide::{Bot, dptree};
use teloxide::dispatching::{dialogue, HandlerExt, UpdateFilterExt, UpdateHandler};
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::{Message, Requester, Update};
use teloxide::utils::command::BotCommands;

use crate::bot::{HandlerResult, MyDialogue, State};
use crate::bot::core::db::client::DatabaseClient;
use crate::bot::handlers::alias::{receive_alias, receive_product_selection, start_add_alias};
use crate::bot::handlers::register::register;
use crate::bot::handlers::search::{receive_search_query, search};

/// These commands are supported:
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub(crate) enum BasicCommands {
    #[command(description = "Display this text")]
    Help,
    #[command(description = "Register with this bot")]
    Start(String),
    #[command(description = "Cancel a dialogue.")]
    Cancel,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum UserCommands {
    #[command(description = "Add mail alias.")]
    Alias,
    #[command(description = "Search for aliases")]
    Search,
}

pub(crate) fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;

    let basic_handler = teloxide::filter_command::<BasicCommands, _>()
        .branch(
            case![State::Start]
                .branch(case![BasicCommands::Help].endpoint(help))
                .branch(case![BasicCommands::Start(token)].endpoint(register))
        )
        .branch(case![BasicCommands::Cancel].endpoint(cancel));

    let command_handler = Update::filter_message()
        .branch(
            case![State::Start]
                .branch(
                    dptree::filter(|database_client: DatabaseClient, msg: Message| {
                    msg.from().map(|user| database_client.known_user_exists(user.id.0 as i64)).unwrap_or(false)
                })
                .filter_command::<UserCommands>()
                .branch(case![UserCommands::Search].endpoint(search))
                .branch(case![UserCommands::Alias].endpoint(start_add_alias)),
            )
        );

    let search_handler = Update::filter_message()
        .branch(case![State::Search].endpoint(receive_search_query));

    let message_handler = Update::filter_message()
        .branch(basic_handler)
        .branch(command_handler)
        .branch(search_handler)
        .branch(case![State::AliasReceive].endpoint(receive_alias))
        .branch(dptree::endpoint(invalid_state));

    let callback_query_handler = Update::filter_callback_query().branch(
        case![State::ReceiveProductChoice { full_name }].endpoint(receive_product_selection),
    );

    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(message_handler)
        .branch(callback_query_handler)
}


async fn help(bot: Bot, msg: Message, database_client: DatabaseClient) -> HandlerResult {
    let available_commands = BasicCommands::descriptions().to_string();
    let basic_commands = format!("Basic commands:\n{}", available_commands);

    if database_client.known_user_exists(msg.chat.id.0) {
        let user_commands = UserCommands::descriptions().to_string();
        let response = format!("{}\n\nUser commands:\n{}", basic_commands, user_commands);
        bot.send_message(msg.chat.id, response).await?;
    } else {
        let reply = format!("You are not registered. Please type /start <start_token> to register. {}", basic_commands);
        bot.send_message(msg.chat.id, reply).await?;
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


