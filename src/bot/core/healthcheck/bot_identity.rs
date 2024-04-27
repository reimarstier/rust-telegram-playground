use std::env;

use anyhow::anyhow;
use teloxide::Bot;
use teloxide::prelude::Requester;
use teloxide::types::Me;

use crate::bot::core::bot_config::TELOXIDE_BOT_NAME_KEY;

pub async fn ensure_configured_bot_name_is_valid(bot: &Bot) -> anyhow::Result<Me> {
    // ensure bot name can be loaded from environment
    let configured_bot_name = env::var(TELOXIDE_BOT_NAME_KEY)
        .map_err(|_error| anyhow!("Bot name must be set in env {}", TELOXIDE_BOT_NAME_KEY))?;

    // compare to telegram identity when authenticated with token
    let result_me = bot.get_me().await;
    match result_me {
        Ok(me) => {
            let bot_username = me.username();
            if bot_username.ne(&configured_bot_name) {
                Err(anyhow!("Could not match username of bot. {}={}, bot username={}", TELOXIDE_BOT_NAME_KEY, configured_bot_name, bot_username))
            } else {
                Ok(me)
            }
        }
        Err(error) => {
            Err(anyhow!("Error getting bot info: {:?}", error))
        }
    }
}