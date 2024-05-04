use anyhow::anyhow;
use diesel::{QueryDsl, RunQueryDsl, SelectableHelper, SqliteConnection};
use diesel::r2d2::ConnectionManager;
use r2d2::PooledConnection;

use crate::bot::core::db::client::DatabaseClient;
use crate::bot::core::db::model::{TelegramAccount, User};
use crate::bot::core::db::schema::{telegram_accounts, users};
use crate::bot::core::db::user_representation::UserRepresentation;

impl DatabaseClient {
    pub(crate) async fn list_users_with_telegram_account(&self, connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>) -> anyhow::Result<Vec<UserRepresentation>> {
        Ok(telegram_accounts::table
            .inner_join(users::table)
            .select((TelegramAccount::as_select(), User::as_select()))
            .load::<(TelegramAccount, User)>(connection)?
            .iter().map(|(account, user)| {
            let account: Option<TelegramAccount> = Some(TelegramAccount { id: account.id, user_id: account.user_id });
            UserRepresentation::from_user(user, &account)
        }).collect::<Vec<_>>())
    }


    pub(crate) fn known_user_exists(&self, telegram_user_id: i64) -> bool {
        let user_ids = self.user_ids.read();
        match user_ids {
            Ok(user_ids) => {
                tracing::trace!("Checking if user is known: {}", telegram_user_id);
                user_ids.get(&telegram_user_id).is_some()
            }
            Err(error) => {
                tracing::error!("Failed to check if user is known: {}. Error: {}", telegram_user_id, error);
                false
            }
        }
    }

    pub(crate) fn known_admin_user_exists(&self, telegram_user_id: i64) -> bool {
        let user = self.known_user(telegram_user_id);
        match user {
            None => {
                false
            }
            Some(user) => {
                user.is_admin()
            }
        }
    }

    pub(crate) fn known_user(&self, telegram_user_id: i64) -> Option<UserRepresentation> {
        let user_ids = self.user_ids.read();
        match user_ids {
            Ok(user_ids) => {
                tracing::trace!("Checking if user is known: {}", telegram_user_id);
                user_ids.get(&telegram_user_id).cloned()
            }
            Err(_) => {
                tracing::error!("Failed to check if user is known: {}", telegram_user_id);
                None
            }
        }
    }

    pub async fn list_registered_users(&self) -> anyhow::Result<Vec<UserRepresentation>> {
        let connection = &mut self.database.get().await?;
        self.list_users_with_telegram_account(connection).await
    }
    pub async fn list_users(&self) -> anyhow::Result<Vec<UserRepresentation>> {
        let connection = &mut self.database.get().await?;

        Ok(users::table
            .left_join(telegram_accounts::table)
            .select((User::as_select(), Option::<TelegramAccount>::as_select()))
            .load::<(User, Option<TelegramAccount>)>(connection)
            .map_err(|error| anyhow!("Error loading users. {}", error))?
            .iter().map(|(user, account)| UserRepresentation::from_user(user, account)).collect::<Vec<_>>())
    }

    pub async fn list_telegram_accounts(&self) -> anyhow::Result<Vec<TelegramAccount>> {
        let connection = &mut self.database.get().await?;

        telegram_accounts::dsl::telegram_accounts
            .select(TelegramAccount::as_select())
            .load(connection)
            .map_err(|error| anyhow!("Error loading accounts. {}", error))
    }
}