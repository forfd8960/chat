use anyhow::Result;

use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

use notify_server::get_router;

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let addr = "0.0.0.0:8989";

    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on: {}", addr);

    let app = get_router();
    axum::serve(listener, app.into_make_service()).await?;

    anyhow::Ok(())
}
