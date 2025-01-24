use anyhow::Result;
use iroh::NodeId;
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

#[wasm_bindgen]
pub async fn accept() -> Result<(), JsError> {
    rust_accept().await.map_err(|e| {
        tracing::error!("Error occured: {e:#?}\n{:#?}", e.backtrace());
        JsError::new(&e.to_string())
    })
}

const ECHO_ALPN: &[u8] = b"iroh-example/echo/0";

#[tracing::instrument]
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

#[tracing::instrument]
pub async fn rust_accept() -> Result<()> {
    let endpoint = iroh::Endpoint::builder()
        .discovery_n0()
        .alpns(vec![ECHO_ALPN.to_vec()])
        .bind()
        .await?;

    tracing::info!("Accepting connections at {}", endpoint.node_id());

    let connecting = loop {
        let Some(incoming) = endpoint.accept().await else {
            return Ok(());
        };
        if let Ok(connecting) = incoming.accept() {
            break connecting;
        }
        tracing::info!("Ignoring incoming connection");
    };

    let connection = connecting.await?;
    let node_id = iroh::endpoint::get_remote_node_id(&connection)?;
    tracing::info!("accepted connection from {node_id}");

    let (mut send, mut recv) = connection.accept_bi().await?;
    let mut bytes_sent = 0;
    while let Some(chunk) = recv.read_chunk(10_000, true).await? {
        bytes_sent += chunk.bytes.len();
        send.write_chunk(chunk.bytes).await?;
    }
    send.finish()?;
    tracing::info!("Copied over {bytes_sent} byte(s)");

    connection.closed().await;

    Ok(())
}
