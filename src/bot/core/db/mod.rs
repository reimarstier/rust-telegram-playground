pub(crate) mod model;
pub(crate) mod schema;
pub(crate) mod connection;
pub(crate) mod admin;
pub(crate) mod client;


#[derive(thiserror::Error, Clone, Debug)]
pub enum DatabaseError {
    #[error("UnknownUser: {0}")]
    UnknownUser(String),
    #[error("CreateError: {0}")]
    CreateError(String),
    #[error("DatabaseError: {0}")]
    Other(String),
}
