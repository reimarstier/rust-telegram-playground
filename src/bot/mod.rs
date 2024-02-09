use teloxide::{
    dispatching::dialogue::InMemStorage,
    prelude::*,
};

pub(crate) mod core;
pub(crate) mod start;
pub(crate) mod handlers;
pub(crate) mod schema;
pub(crate) mod admin;

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    Search,
    AliasReceive,
    ReceiveProductChoice {
        full_name: String,
    },
}
