use anyhow::Result;

#[tokio::main]
pub async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    Ok(())
}
