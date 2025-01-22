use anyhow::Result;
use iroh::NodeId;
use iroh_base::{RelayUrl, SecretKey};
use iroh_relay::client::ClientBuilder;
use std::str::FromStr;
use tracing::level_filters::LevelFilter;
use tracing_subscriber_wasm::MakeConsoleWriter;
use wasm_bindgen::{prelude::wasm_bindgen, JsError};

#[wasm_bindgen(start)]
fn start() {
    console_error_panic_hook::set_once();

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
}

#[wasm_bindgen]
pub async fn connect(node_id: String) -> Result<(), JsError> {
    rust_connect(node_id).await.map_err(|e| {
        tracing::error!("Error occured: {e:#?}\n{:#?}", e.backtrace());
        JsError::new(&e.to_string())
    })
}

const ECHO_ALPN: &[u8] = b"iroh-example/echo/0";

pub async fn rust_connect(node_id: String) -> Result<()> {
    let endpoint = iroh::Endpoint::builder()
        .discovery_n0()
        .alpns(vec![ECHO_ALPN.to_vec()])
        .bind()
        .await?;

    let node_id = NodeId::from_str(&node_id)?;

    let conn = endpoint.connect(node_id, ECHO_ALPN).await?;
    tracing::info!("Connected!");
    let (mut send, mut recv) = conn.open_bi().await?;

    send.write_all(b"Hello, world!").await?;
    send.finish()?;
    tracing::info!("Sent");

    let response = recv.read_to_end(1000).await?;
    let resp_str = String::from_utf8_lossy(&response);
    tracing::info!(?resp_str, "Received!");

    conn.close(0u32.into(), b"bye!");
    conn.closed().await;

    tracing::info!("Done :)");

    endpoint.close().await;

    Ok(())
}
