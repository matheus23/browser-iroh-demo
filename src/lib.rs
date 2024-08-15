use std::str::FromStr;

use anyhow::Result;
use futures_lite::stream::StreamExt;
use iroh_net::{ticket::NodeTicket, Endpoint};
use tracing::level_filters::LevelFilter;
use tracing_subscriber_wasm::MakeConsoleWriter;
use wasm_bindgen::prelude::wasm_bindgen;

const PING_ALPN: &[u8] = b"test/ping/0";

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
pub async fn do_a_ping_js(node_id: String) -> bool {
    do_a_ping(node_id).await.unwrap();
    true
}

#[wasm_bindgen]
pub async fn serve_pongs_js() {
    serve_pongs().await.unwrap();
}

pub async fn do_a_ping(ticket: String) -> Result<()> {
    tracing::info!("do_a_ping()");

    let endpoint = endpoint().await?;

    tracing::info!("Connecting to {ticket}");

    let node_addr = NodeTicket::from_str(&ticket)?.node_addr().clone();

    tracing::info!(?node_addr.info, "Got node info");

    let conn = endpoint.connect(node_addr, &PING_ALPN).await?;
    tracing::info!("Connected! Opening a channel.");

    let (mut send, mut recv) = conn.open_bi().await?;
    tracing::info!("Channel opened, sending ping");

    let mut ping = vec![0u8; 8];
    getrandom::getrandom(&mut ping)?;

    send.write_all(&ping).await?;
    send.finish().await?;
    tracing::info!("Sent ping. Waiting for response.");
    let pong = recv.read_to_end(1000).await?;

    if ping == pong {
        tracing::info!("Received correct pong");
    } else {
        tracing::error!("Ping failed");
    }

    Ok(())
}

pub async fn endpoint() -> Result<Endpoint> {
    Ok(Endpoint::builder()
        .alpns(vec![PING_ALPN.to_vec()])
        .bind(0)
        .await?)
}

pub async fn serve_pongs() -> Result<()> {
    tracing::info!("serve_pongs()");

    let endpoint = endpoint().await?;

    tracing::info!("Waiting for home relay");
    let relay = endpoint.watch_home_relay().next().await;
    tracing::info!(?relay, "Got home relay");

    let ticket = NodeTicket::new(endpoint.node_addr())?.to_string();

    tracing::info!("Node address: {}", ticket);

    while let Some(mut connecting) = endpoint.accept().await {
        tracing::info!("Incoming connection");
        match connecting.alpn().await?.as_ref() {
            PING_ALPN => {
                let conn = connecting.await?;

                let node_id = iroh_net::endpoint::get_remote_node_id(&conn)?;
                tracing::info!("Connection from {node_id}. Waiting for stream.");

                let (mut send, mut recv) = conn.accept_bi().await?;
                tracing::info!("Stream opened. Reading ping.");
                let ping = recv.read_to_end(1000).await?;
                tracing::info!("Got ping {}", hex::encode(&ping));
                send.write_all(&ping).await?;
                send.finish().await?;
                tracing::info!("Sent pong back");
            }
            unknown => tracing::warn!("Unknown ALPN: {}", String::from_utf8_lossy(unknown)),
        }
    }

    tracing::info!("Closing endpoint.");

    Ok(())
}
