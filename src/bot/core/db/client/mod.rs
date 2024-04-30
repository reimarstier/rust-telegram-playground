use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use diesel::{QueryDsl, RunQueryDsl, SqliteConnection};
use diesel::ExpressionMethods;
use diesel::r2d2::ConnectionManager;
use r2d2::PooledConnection;

use crate::bot::core::db::connection::MyDatabaseConnection;
use crate::bot::core::db::DatabaseError;
use crate::bot::core::db::model::{TelegramAccount, User};
use crate::bot::core::db::schema::{telegram_accounts, users};
use crate::bot::core::db::user_representation::UserRepresentation;

pub mod admin_client;
mod list_client;

#[derive(Debug, Clone)]
pub(crate) struct DatabaseClient {
    /// Map of telegram user id to user representation
    user_ids: Arc<RwLock<HashMap<i64, UserRepresentation>>>,
    database: MyDatabaseConnection,
}

impl DatabaseClient {
    pub(crate) async fn load(database: MyDatabaseConnection) -> Result<Self, anyhow::Error> {
        let connection = &mut database.get().await?;
        let user_ids = Default::default();

        let mut client = Self {
            user_ids,
            database,
        };
        client.update_user_hash_map(connection).await?;
        Ok(client)
    }

    async fn update_user_hash_map(&mut self, connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>) -> anyhow::Result<()> {
        let user_list = self.list_users_with_telegram_account(connection).await?;
        tracing::debug!("Updating user hash map.");
        let user_ids = self.user_ids.write();
        match user_ids {
            Ok(mut user_ids) => {
                for user in user_list {
                    let telegram_id = user.telegram_id.expect("Only existing telegram accounts were listed.");
                    user_ids.insert(telegram_id, user);
                }
            }
            Err(error) => {
                tracing::error!("Failed to lock user hash map. {}", error);
            }
        }


        Ok(())
    }

    fn get_user_by_name(&self, connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>, user_name: &str) -> Result<UserRepresentation, DatabaseError> {
        users::table
            .filter(users::name.eq(user_name))
            .left_outer_join(telegram_accounts::table)
            .first::<(User, Option<TelegramAccount>)>(connection)
            .map(|(user, account)| {
                UserRepresentation::from_user(&user, &account)
            })
            .map_err(|error|
                DatabaseError::UnknownUser(format!("Could not find user with name {}. Error: {}", user_name, error))
            )
    }


}
