use rand::Rng;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt::writer::MakeWriterExt;

use crate::bot::core::bot_config::storage::BotStorageConfig;
use crate::MyResult;

pub fn init_tracing() -> MyResult {
    let storage_config = BotStorageConfig::new()?;

    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .max_log_files(14)
        .filename_prefix("telegrambot")
        .filename_suffix("log")
        .build(storage_config.log_directory)
        .expect("initializing rolling file appender failed");

    let stdout = std::io::stdout
        .with_max_level(tracing::Level::TRACE);

    tracing_subscriber::fmt()
        .with_writer(stdout.and(file_appender))
        .compact()
        .init();

    Ok(())
}

pub fn random_start_token() -> String {
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}
