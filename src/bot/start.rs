use teloxide::{Bot, dptree};
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::dispatching::Dispatcher;
use teloxide::error_handlers::LoggingErrorHandler;
use teloxide::types::Me;
use teloxide::update_listeners::webhooks::Options;

use crate::bot::core::bot_config::BotConfig;
use crate::bot::core::bot_config::webhook::BotConfigWebHook;
use crate::bot::core::db::client::DatabaseClient;
use crate::bot::core::db::connection::MyDatabaseConnection;
use crate::bot::core::dispatch::axum_update_listener;
use crate::bot::core::healthcheck::bot_identity::ensure_configured_bot_name_is_valid;
use crate::bot::schema::schema;
use crate::bot::State;
use crate::{build, MyResult};

pub(crate) async fn bot_start(use_webhook: bool) -> MyResult {
    let bot = Bot::from_env();
    let me = ensure_configured_bot_name_is_valid(&bot).await?;
    log::info!("Bot started: {:?}", me);
    print_banner(me);

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

fn print_banner(me: Me) {
    let now = chrono::offset::Utc::now();
    // https://patorjk.com/software/taag/#p=display&f=Big&t=Telegrambot
    println!(r"
  _______   _                                _           _
 |__   __| | |                              | |         | |
    | | ___| | ___  __ _ _ __ __ _ _ __ ___ | |__   ___ | |_
    | |/ _ \ |/ _ \/ _` | '__/ _` | '_ ` _ \| '_ \ / _ \| __|
    | |  __/ |  __/ (_| | | | (_| | | | | | | |_) | (_) | |_
    |_|\___|_|\___|\__, |_|  \__,_|_| |_| |_|_.__/ \___/ \__|
                    __/ |
                   |___/

    Bot name:   {}
    Bot url:    https://t.me/{}
    Version:    {}
    Git Hash:   {}
    Git Date:   {}
    Build date: {}
    Start date: {}

", me.username(), me.username(),
             build::PKG_VERSION, build::COMMIT_HASH, build::COMMIT_DATE, build::BUILD_TIME, now)
}
