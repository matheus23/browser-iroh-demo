use std::str::FromStr;

use anyhow::Result;
use futures_lite::stream::StreamExt;
use iroh_base::SecretKey;
use iroh_relay::client::ClientBuilder;
use tracing::level_filters::LevelFilter;
use tracing_subscriber_wasm::MakeConsoleWriter;
use wasm_bindgen::{prelude::wasm_bindgen, JsError};

#[wasm_bindgen(start)]
fn start() {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::TRACE)
        .with_writer(
            // To avoide trace events in the browser from showing their JS backtrace
            MakeConsoleWriter::default().map_trace_level_to(tracing::Level::DEBUG),
        )
        // If we don't do this in the browser, we get a runtime error.
        .without_time()
        .with_ansi(false)
        .init();

    tracing::info!("(testing logging) Logging setup");

    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub async fn connect_relay() -> Result<(), JsError> {
    rust_connect_relay()
        .await
        .map_err(|e| JsError::new(&e.to_string()))
}

pub async fn rust_connect_relay() -> Result<()> {
    let mut key = [0u8; 32];
    getrandom::getrandom(&mut key)?;
    let secret_key = SecretKey::from_bytes(&key);
    let url: url::Url = "https://euw1-1.relay.iroh.network.".parse()?;
    let client = ClientBuilder::new(url, secret_key, ()).connect().await?;

    tracing::info!("Connected :)");

    Ok(())
}
