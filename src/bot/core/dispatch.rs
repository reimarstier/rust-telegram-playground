use std::convert::Infallible;
use teloxide::prelude::Requester;
use teloxide::update_listeners::UpdateListener;
use teloxide::update_listeners::webhooks::{axum_to_router, Options};
use crate::bot::core::healthcheck::endpoint::healthcheck_endpoint;

pub async fn axum_update_listener<R>(
    bot: R,
    options: Options,
) -> Result<impl UpdateListener<Err = Infallible>, R::Err>
    where
        R: Requester + Send + 'static,
        <R as Requester>::DeleteWebhook: Send,
{
    let Options { address, .. } = options;

    let (mut update_listener, stop_flag, app) = axum_to_router(bot, options).await?;
    let my_router = axum::Router::new()
        .route("/healthcheck", axum::routing::get(healthcheck_endpoint))
        .fallback_service(app);

    let stop_token = update_listener.stop_token();

    tokio::spawn(async move {
        axum::Server::bind(&address)
            .serve(my_router.into_make_service())
            .with_graceful_shutdown(stop_flag)
            .await
            .map_err(|err| {
                stop_token.stop();
                err
            })
            .expect("Axum server error");
    });

    Ok(update_listener)
}