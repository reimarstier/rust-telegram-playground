use diesel::{Associations, Identifiable, Insertable, Queryable, Selectable};

use crate::bot::core::db::schema::telegram_accounts;
use crate::bot::core::db::schema::users;

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub name: &'a str,
    pub start: &'a str,
}


#[derive(Queryable, Selectable, Identifiable, PartialEq, Debug)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
    pub id: i64,
    pub name: String,
    pub start: String,
}

#[derive(Insertable)]
#[diesel(table_name = telegram_accounts)]
pub struct NewTelegramAccount<'a> {
    pub id: &'a i64,
    pub user_id: &'a i64,
}

#[derive(Queryable, Selectable, Identifiable, Associations, PartialEq, Debug)]
#[diesel(table_name = telegram_accounts)]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct TelegramAccount {
    pub id: i64,
    pub user_id: i64,
}
