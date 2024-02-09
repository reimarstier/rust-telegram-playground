use diesel::{BelongingToDsl, QueryDsl, QueryResult, RunQueryDsl, SelectableHelper, SqliteConnection};
use diesel::ExpressionMethods;
use crate::bot::core::db::DatabaseError;
use crate::bot::core::db::model::{NewTelegramAccount, NewUser, TelegramAccount, User};
use crate::bot::core::db::schema::{telegram_accounts, users};
use crate::bot::core::db::schema::users::start;
use crate::bot::core::util::random_start_token;

pub fn create_user(conn: &mut SqliteConnection, user_name: &str) -> QueryResult<User> {
    let start_token = random_start_token();
    let new_user = NewUser { name: user_name, start: &start_token };
    diesel::insert_into(users::table)
        .values(&new_user)
        .returning(User::as_returning())
        .get_result(conn)
}


pub fn register_telegram_account_of_user(conn: &mut SqliteConnection, start_token: &str, telegram_id: &i64) -> Result<TelegramAccount, DatabaseError> {
    let user = users::table
        .filter(start.eq(start_token))
        .first::<User>(conn)
        .map_err(|error|
            DatabaseError::UnknownUser(format!("Could not find token '{}' for user id '{}'. Error: {}", start_token, telegram_id, error))
        )?;

    let account = TelegramAccount::belonging_to(&user)
        .select(TelegramAccount::as_select())
        .first::<TelegramAccount>(conn);
    match account {
        Ok(account) => {
            Ok(account)
        }
        Err(error) => {
            match error {
                diesel::result::Error::NotFound => {
                    let new_account = NewTelegramAccount { id: telegram_id, user_id: &user.id };
                    let account = diesel::insert_into(telegram_accounts::table)
                        .values(&new_account)
                        .returning(TelegramAccount::as_returning())
                        .get_result(conn)
                        .map_err(|error|
                            DatabaseError::CreateError(format!("Could not find token '{}' for user id '{}'. Error: {}", start_token, telegram_id, error))
                        )?;

                    Ok(account)
                }
                _ => {
                    Err(DatabaseError::Other(format!("Could not telegram account for user: {:?}. Error: {}", user, error)))
                }
            }
        }
    }
}
