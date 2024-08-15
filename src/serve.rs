use anyhow::Result;
use browser_iroh_net_demo::serve_pongs;

#[tokio::main]
pub async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    serve_pongs().await?;

    Ok(())
}
