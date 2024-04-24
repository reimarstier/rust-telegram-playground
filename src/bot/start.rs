use std::env;
use anyhow::anyhow;
use teloxide::{Bot, dptree};
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::dispatching::Dispatcher;
use teloxide::error_handlers::LoggingErrorHandler;
use teloxide::prelude::Requester;
use teloxide::update_listeners::webhooks::Options;

use crate::bot::core::bot_config::{BotConfig, TELOXIDE_BOT_NAME_KEY};
use crate::bot::core::bot_config::webhook::BotConfigWebHook;
use crate::bot::core::db::client::DatabaseClient;
use crate::bot::core::db::connection::MyDatabaseConnection;
use crate::bot::core::dispatch::axum_update_listener;
use crate::bot::schema::schema;
use crate::bot::State;
use crate::MyResult;

pub(crate) async fn bot_start(use_webhook: bool) -> MyResult {
    let bot = Bot::from_env();
    let result_me = bot.get_me().await;
    match result_me {
        Ok(me) => {
            log::info!("Bot started: {:?}", me);
        }
        Err(error) => {
            log::error!("Error getting bot info: {:?}", error);
            return Err(anyhow!("Error getting bot info: {:?}", error));
        }
    }
    // ensure bot name can be loaded
    let _bot_name = env::var(TELOXIDE_BOT_NAME_KEY)
        .map_err(|_error| anyhow!("Bot name must be set in env {}", TELOXIDE_BOT_NAME_KEY))?;

    let database_connection = MyDatabaseConnection::new().await?;
    let database_client = DatabaseClient::load(database_connection.clone()).await?;
    let bot_config = BotConfig::new()?;


    let dependency_map = dptree::deps![InMemStorage::<State>::new(), database_client, bot_config];

    if use_webhook {
        log::info!("Starting bot using webhook listener...");
        let webhook_config = BotConfigWebHook::new()?;
        log::info!("Webhook config: {:?}", webhook_config);

        let listener = axum_update_listener(
            bot.clone(),
            Options::new(webhook_config.socket_address, webhook_config.public_bot_url),
        ).await
            .expect("Couldn't setup webhook");
        Dispatcher::builder(bot, schema())
            .dependencies(dependency_map)
            .enable_ctrlc_handler()
            .build()
            .dispatch_with_listener(
                listener,
                LoggingErrorHandler::with_custom_text("An error from the update listener"),
            )
            .await;
    } else {
        log::info!("Starting bot without webhook listener...");
        Dispatcher::builder(bot, schema())
            .dependencies(dependency_map)
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;
    }

    Ok(())
}
