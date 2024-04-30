use crate::bot::core::db::client::admin_client::DatabaseAdminClient;
use crate::bot::core::db::client::DatabaseClient;
use crate::bot::core::db::connection::MyDatabaseConnection;
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
    /// Delete a user
    Delete { user_name: String },
    /// Link telegram id to user account
    AddTelegram { start_token: String, telegram_id: i64 },
}

const PRINT_BARRIER: &str = "----------------------------------------";

fn print_header(msg: &str, with_newline: bool) {
    if with_newline {
        println!();
    }
    println!("{}", PRINT_BARRIER);
    println!("{}", msg);
    println!("{}", PRINT_BARRIER);
}

impl AdminCli {
    pub(crate) async fn default_handling(&self) -> MyResult {
        let database_connection = MyDatabaseConnection::new().await?;
        let mut database_client = DatabaseClient::load(database_connection.clone()).await?;

        match &self.task {
            TaskCli::Show => {
                print_header("List all users:", false);
                let all_users = database_client.list_users().await?;
                for user in all_users {
                    println!("{}", user);
                }

                print_header("List all telegram accounts:", true);
                let telegram_accounts = database_client.list_telegram_accounts().await?;
                for account in telegram_accounts {
                    println!("id={}: user_id={}", account.id, account.user_id);
                }
            }
            TaskCli::Add { user_name } => {
                let user = database_client.create_user(user_name).await?;
                println!("Created user {}", user);
            }
            TaskCli::Delete { user_name } => {
                let user = database_client.delete_user(user_name).await?;
                println!("Deleted user {}", user);
            }
            TaskCli::AddTelegram { start_token, telegram_id } => {
                let result = database_client.register_telegram_account_of_user(start_token, *telegram_id).await?;
                println!("{:?}", result);
            }
        }
        Ok(())
    }
}

