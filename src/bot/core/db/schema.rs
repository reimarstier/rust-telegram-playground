// @generated automatically by Diesel CLI.

diesel::table! {
    telegram_accounts (id) {
        id -> BigInt,
        user_id -> BigInt,
    }
}

diesel::table! {
    users (id) {
        id -> BigInt,
        name -> Text,
        start -> Text,
        role -> Text,
    }
}

diesel::joinable!(telegram_accounts -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    telegram_accounts,
    users,
);
