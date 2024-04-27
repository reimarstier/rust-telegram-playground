use rand::Rng;
use tracing::Metadata;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt::writer::MakeWriterExt;

use crate::bot::core::bot_config::storage::BotStorageConfig;
use crate::MyResult;

pub fn init_tracing() -> MyResult {
    let storage_config = BotStorageConfig::new()?;

    fn filter_hyper_logs(metadata: &Metadata) -> bool {
        metadata.module_path().map(|module| {
            !module.contains("hyper")
        }).unwrap_or(true)
    }

    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .max_log_files(14)
        .filename_prefix("telegrambot")
        .filename_suffix("log")
        .build(storage_config.log_directory)
        .expect("initializing rolling file appender failed")
        .with_filter(filter_hyper_logs);

    let stdout = std::io::stdout
        .with_filter(filter_hyper_logs)
        .with_max_level(tracing::Level::DEBUG);

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(stdout.and(file_appender))
        // Could be nice to use ansi coloring in stdout but not in file output for persistence, disabling for now
        .with_ansi(false)
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
