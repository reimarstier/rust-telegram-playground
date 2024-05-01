use diesel::{QueryDsl, RunQueryDsl, SelectableHelper, SqliteConnection};
use crate::bot::core::db::DatabaseError;
use crate::bot::core::db::model::{NewTelegramAccount, NewUser, TelegramAccount, User};
use crate::bot::core::db::schema::{telegram_accounts, users};
use crate::bot::core::db::user_representation::{UserRepresentation, UserRole};
use crate::bot::core::util::random_start_token;
use diesel::ExpressionMethods;
use diesel::r2d2::ConnectionManager;
use r2d2::PooledConnection;
use crate::bot::core::db::client::DatabaseClient;

pub trait DatabaseAdminClient {
    async fn create_user(&self, user_name: &str) -> Result<UserRepresentation, DatabaseError>;
    async fn delete_user(&self, user_name: &str) -> Result<UserRepresentation, DatabaseError>;
    async fn register_telegram_account_of_user(&mut self, start_token: &str, telegram_id: i64) -> Result<UserRepresentation, DatabaseError>;
}

impl DatabaseAdminClient for DatabaseClient {
    async fn create_user(&self, user_name: &str) -> Result<UserRepresentation, DatabaseError> {
        let connection = &mut self.database.get().await
            .map_err(|error| DatabaseError::Connection(format!("when creating user: {}", error)))?;

        let start_token = random_start_token();
        let user = UserRole::User.to_string();
        let new_user = NewUser { name: user_name, start: &start_token, role: &user };
        diesel::insert_into(users::table)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result(connection)
            .map(|user| {
                let account = None;
                UserRepresentation::from_user(&user, &account)
            })
            .map_err(|error| DatabaseError::CreateError(format!("Could not create user '{}'. {}", user_name, error)))
    }

    async fn delete_user(&self, user_name: &str) -> Result<UserRepresentation, DatabaseError> {
        let connection = &mut self.database.get().await
            .map_err(|error| DatabaseError::Connection(format!("when deleting user: {}", error)))?;

        let user = self.get_user_by_name(connection, user_name);
        match user {
            Ok(user) => {
                if let Some(telegram_id) = user.telegram_id {
                    diesel::delete(telegram_accounts::table)
                        .filter(telegram_accounts::id.eq(&telegram_id))
                        .execute(connection)
                        .map_err(|error| DatabaseError::DeleteError(format!("Could not delete user '{}'. {}", user_name, error)))?;
                };
                let result = diesel::delete(users::table)
                    .filter(users::name.eq(user_name))
                    .execute(connection)
                    .map_err(|error| DatabaseError::DeleteError(format!("Could not delete user '{}'. {}", user_name, error)));
                match result {
                    Ok(size) => {
                        assert!(size.eq(&1));
                        Ok(user)
                    }
                    Err(error) => {
                        Err(error)
                    }
                }
            }
            Err(error) => {
                Err(error)
            }
        }
    }
    async fn register_telegram_account_of_user(&mut self, start_token: &str, telegram_id: i64) -> Result<UserRepresentation, DatabaseError> {
        match self.known_user(telegram_id) {
            Some(user) => {
                Ok(user)
            }
            None => {
                let connection = &mut self.database.get().await
                    .map_err(|error| DatabaseError::Connection(format!("while registering: {}", error)))?;

                tracing::debug!("Telegram account does not exist yet, check if start token={} is present in user table. telegram id={}.", start_token, telegram_id);
                let user = self.get_user_with_start_token(connection, start_token, telegram_id)?;

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

impl DatabaseClient {
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

    fn get_user_with_start_token(&self, connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>, start_token: &str, telegram_id: i64) -> Result<UserRepresentation, DatabaseError> {
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
}