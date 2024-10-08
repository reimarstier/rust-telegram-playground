use std::convert::Infallible;
use teloxide::prelude::Requester;
use teloxide::update_listeners::UpdateListener;
use teloxide::update_listeners::webhooks::{axum_to_router, Options};
use crate::bot::core::bot_config::TELEGRAM_BOT_ENDPOINT_HEALTHCHECK;
use crate::bot::core::healthcheck::endpoint::healthcheck_endpoint;

pub async fn axum_update_listener<R>(
    bot: R,
    options: Options,
) -> Result<impl UpdateListener<Err = Infallible>, R::Err>
    where
        R: Requester + Send + 'static,
        <R as Requester>::DeleteWebhook: Send,
{
    // loosely derived from: teloxide: src/update_listeners/webhooks/axum.rs
    let Options { address, .. } = options;

    let (mut update_listener, stop_flag, app) = axum_to_router(bot, options).await?;
    let my_router = axum::Router::new()
        .route(TELEGRAM_BOT_ENDPOINT_HEALTHCHECK, axum::routing::get(healthcheck_endpoint))
        .fallback_service(app);

    let stop_token = update_listener.stop_token();

    tokio::spawn(async move {
        let tcp_listener = tokio::net::TcpListener::bind(address)
            .await
            .inspect_err(|_err| {
                stop_token.stop();
            })
            .expect("Couldn't bind to the address");

        axum::serve(tcp_listener, my_router)
            .with_graceful_shutdown(stop_flag)
            .await
            .inspect_err(|_err| {
                stop_token.stop();
            })
            .expect("Axum server error");
    });

    Ok(update_listener)
}