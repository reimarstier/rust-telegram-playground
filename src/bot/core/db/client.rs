use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use anyhow::anyhow;
use diesel::{QueryDsl, RunQueryDsl, SelectableHelper, SqliteConnection};
use diesel::ExpressionMethods;
use diesel::r2d2::ConnectionManager;
use r2d2::PooledConnection;

use crate::bot::core::db::connection::MyDatabaseConnection;
use crate::bot::core::db::DatabaseError;
use crate::bot::core::db::model::{NewTelegramAccount, NewUser, TelegramAccount, User};
use crate::bot::core::db::schema::{telegram_accounts, users};
use crate::bot::core::db::user_representation::UserRepresentation;
use crate::bot::core::util::random_start_token;

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

    async fn list_users_with_telegram_account(&self, connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>) -> anyhow::Result<Vec<UserRepresentation>> {
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

    pub async fn create_user(&self, user_name: &str) -> Result<User, DatabaseError> {
        let connection = &mut self.database.get().await
            .map_err(|error| DatabaseError::Connection(format!("when creating user: {}", error)))?;

        let start_token = random_start_token();
        let new_user = NewUser { name: user_name, start: &start_token };
        diesel::insert_into(users::table)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result(connection)
            .map_err(|error| DatabaseError::UnknownUser(format!("Could not find user '{}'. {}", user_name, error)))
    }

    fn user_with_start_token(&self, connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>, start_token: &str, telegram_id: i64) -> Result<UserRepresentation, DatabaseError> {
        users::table
            .filter(users::start.eq(start_token))
            .left_outer_join(telegram_accounts::table)
            .first::<(User, Option<TelegramAccount>)>(connection)
            .map(|(user, account)| {
                UserRepresentation::from_user(&user, &account)
            })
            .map_err(|error|
                DatabaseError::UnknownUser(format!("Could not find token '{}' for telegram id '{}'. Error: {}", start_token, telegram_id, error))
            )
    }

    async fn add_new_telegram_account(&mut self, connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>, start_token: &str, user_id: i64, telegram_id: i64) -> Result<TelegramAccount, DatabaseError> {
        let new_account = NewTelegramAccount { id: &telegram_id, user_id: &user_id };
        let result = diesel::insert_into(telegram_accounts::table)
            .values(&new_account)
            .returning(TelegramAccount::as_returning())
            .get_result(connection)
            .map_err(|error|
                DatabaseError::CreateError(format!("Could not find token '{}' for telegram id '{}'. Error: {}", start_token, telegram_id, error))
            );
        match result {
            Ok(account) => {
                self.update_user_hash_map(connection).await
                    .map_err(|error|
                        DatabaseError::CreateError(format!("Could not update user hash map after creating tg_id={}. Error: {}", telegram_id, error))
                    )?;
                Ok(account)
            }
            Err(error) => {
                Err(error)
            }
        }
    }

    pub async fn register_telegram_account_of_user(&mut self, start_token: &str, telegram_id: i64) -> Result<UserRepresentation, DatabaseError> {
        match self.known_user(telegram_id) {
            Some(user) => {
                Ok(user)
            }
            None => {
                let connection = &mut self.database.get().await
                    .map_err(|error| DatabaseError::Connection(format!("while registering: {}", error)))?;

                tracing::debug!("Telegram account does not exist yet, check if start token={} is present in user table. telegram id={}.", start_token, telegram_id);
                let user = self.user_with_start_token(connection, start_token, telegram_id)?;

                match user.telegram_id {
                    Some(present_telegram_id) => {
                        if present_telegram_id.eq(&telegram_id) {
                            // this case should not be possible because of the match above
                            tracing::debug!("registered user has used the start token again. telegram id={}", telegram_id);
                            Ok(user)
                        } else {
                            tracing::warn!("registered user has used the start token with a new telegram id, refusing to update/interact. telegram id={}", telegram_id);
                            Err(DatabaseError::Other(format!("Telegram id={} does not match the found telegram id={}", telegram_id, present_telegram_id)))
                        }
                    }
                    None => {
                        // received valid start token, creating new telegram account link
                        let account = self.add_new_telegram_account(connection, start_token, user.id, telegram_id).await?;
                        assert!(account.id.eq(&telegram_id), "Newly created telegram id should match.");
                        Ok(self.known_user(telegram_id).expect("Newly created user should exist in hash map."))
                    }
                }
            }
        }
    }
}
