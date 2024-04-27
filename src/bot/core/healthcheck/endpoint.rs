use serde::{Deserialize, Serialize};
use axum::Json;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub(crate) struct BotHealthResult {
    pub ok: bool,
}

pub(crate) async fn healthcheck_endpoint() -> Json<BotHealthResult> {
    Json(BotHealthResult { ok: true })
}
