use anyhow::Result;

use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

use chat_server::{get_router, AppConfig};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Hello, world!");
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let conf = AppConfig::load()?;
    let addr = format!("0.0.0.0:{}", conf.server.port);

    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on: {}", addr);

    let app = get_router(conf);

    axum::serve(listener, app.into_make_service()).await?;

    anyhow::Ok(())
}
