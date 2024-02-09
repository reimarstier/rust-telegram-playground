use clap::{ArgAction, Parser, Subcommand};
use dotenvy::dotenv;

use crate::bot::admin::AdminCli;
use crate::bot::core::healthcheck::telegram_healthcheck;
use crate::bot::core::util;
use crate::bot::start::bot_start;

mod bot;

pub type MyResult = anyhow::Result<()>;

#[derive(Parser)]
#[command(name = "telegrambot")]
#[command(about = "a nice bot for telegram")]
#[command(long_version = None)]
struct Cli {
    #[command(subcommand)]
    command: TaskCli,
}

#[derive(Subcommand)]
enum TaskCli {
    /// Run telegrambot
    Bot {
        #[arg(long, action = ArgAction::SetTrue)]
        disable_webhook: bool,
    },
    /// Check telegram API for health of this bot
    Healthcheck,
    /// Admin cli
    Admin(AdminCli),
}


#[tokio::main]
async fn main() -> MyResult {
    dotenv().expect(".env file not found!");

    let args = Cli::parse();
    util::init_tracing()?;

    match args.command {
        TaskCli::Bot { disable_webhook } => {
            if disable_webhook {
                bot_start(false).await?;
            } else {
                bot_start(true).await?;
            }
        }
        TaskCli::Healthcheck => {
            telegram_healthcheck().await?;
        }
        TaskCli::Admin(implementation) => {
            implementation.default_handling().await?;
        }
    }

    Ok(())
}
