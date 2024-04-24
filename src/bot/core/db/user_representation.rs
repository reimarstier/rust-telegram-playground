use std::env;
use std::fmt::{Display, Formatter};

use crate::bot::core::bot_config::TELOXIDE_BOT_NAME_KEY;
use crate::bot::core::db::model::{TelegramAccount, User};

#[derive(PartialEq, Debug, Clone)]
pub struct UserRepresentation {
    pub id: i64,
    pub name: String,
    pub start_token: String,
    pub bot_start_url: String,
    pub telegram_id: Option<i64>
}

impl UserRepresentation {
    pub fn from_user(user: &User, account: &Option<TelegramAccount>) -> Self {
        let bot_name = env::var(TELOXIDE_BOT_NAME_KEY)
            .unwrap_or_else(|_| panic!("Bot name must be set in env {}", TELOXIDE_BOT_NAME_KEY));

        let bot_start_url = format!("https://t.me/{}?start={}", bot_name, user.start);
        let telegram_id = account.as_ref().map(|account| account.id);
        Self {
            id: user.id,
            name: user.name.clone(),
            start_token: user.start.clone(),
            bot_start_url,
            telegram_id,
        }
    }
}

impl Display for UserRepresentation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let output = format!("id={}: name={} start_token={} url={} telegram_id={:?}", self.id, self.name, self.start_token, self.bot_start_url, self.telegram_id);
        f.write_str(&output)
    }
}