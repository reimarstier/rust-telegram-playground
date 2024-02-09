use std::env;
use anyhow::anyhow;
use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};
use crate::bot::core::bot_config::TELOXIDE_BOT_NAME_KEY;

use crate::bot::core::db::admin::{create_user, register_telegram_account_of_user};
use crate::bot::core::db::connection::MyDatabaseConnection;
use crate::bot::core::db::model::{TelegramAccount, User};
use crate::bot::core::db::schema::{telegram_accounts, users};
use crate::MyResult;

/// Manage the bot.
#[derive(clap::Parser)]
pub struct AdminCli {
    #[command(subcommand)]
    pub(crate) task: TaskCli,
}

#[derive(clap::Subcommand)]
pub enum TaskCli {
    /// Show database
    Show,
    /// Add user
    Add { user_name: String },
    /// Add telegram user account
    AddTelegram { start_token: String, telegram_id: i64 },
}

impl AdminCli {
    pub(crate) async fn default_handling(&self) -> MyResult {
        let database = MyDatabaseConnection::new().await?;
        let sqlite_con = &mut database.get().await?;

        match &self.task {
            TaskCli::Show => {
                println!("List all users:");
                let all_users = users::dsl::users
                    .select(User::as_select())
                    .load(sqlite_con)
                    .expect("Error loading users");
                for user in all_users {
                    let bot_name = env::var(TELOXIDE_BOT_NAME_KEY)
                        .map_err(|_error| anyhow!("Bot name must be set in env {}", TELOXIDE_BOT_NAME_KEY))?;
                    let start_url = format!("https://t.me/{}?start={}", bot_name, user.start);
                    println!("{}: name={} start={} url={}", user.id, user.name, user.start, start_url);
                }

                println!("List all telegram accounts:");
                let telegram_accounts = telegram_accounts::dsl::telegram_accounts
                    .select(TelegramAccount::as_select())
                    .load(sqlite_con)
                    .expect("Error loading accounts");
                for account in telegram_accounts {
                    println!("id={}: user_id={}", account.id, account.user_id);
                }
            }
            TaskCli::Add { user_name } => {
                create_user(sqlite_con, user_name)?;
            }
            TaskCli::AddTelegram { start_token, telegram_id } => {
                let result = register_telegram_account_of_user(sqlite_con, start_token, telegram_id)?;
                println!("{:?}", result);
            }
        }
        Ok(())
    }
}

