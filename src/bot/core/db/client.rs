use std::collections::HashMap;

use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};

use crate::bot::core::db::connection::MyDatabaseConnection;
use crate::bot::core::db::model::TelegramAccount;
use crate::bot::core::db::schema::telegram_accounts;

#[derive(Debug, Clone)]
pub(crate) struct DatabaseClient {
    /// Map of telegram user id to user id
    user_ids: HashMap<i64, i64>,
    // TODO: refactor database access to use the client more often
    database: MyDatabaseConnection,
}

impl DatabaseClient {
    pub(crate) async fn load(database: MyDatabaseConnection) -> Result<Self, anyhow::Error> {
        let sqlite_con = &mut database.get().await?;

        let telegram_accounts = telegram_accounts::dsl::telegram_accounts
            .select(TelegramAccount::as_select())
            .load(sqlite_con)?;
        let user_ids = telegram_accounts.iter().map(|account| (account.id, account.user_id)).collect::<HashMap<_, _>>();

        Ok(Self {
            user_ids,
            database,
        })
    }
    pub(crate) fn known_user(&self, chat_id: i64) -> bool {
        tracing::debug!("Checking if user is known: {}", chat_id);
        self.user_ids.get(&chat_id).is_some()
    }
}
