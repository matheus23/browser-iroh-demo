use anyhow::Result;
use iroh_base::{RelayUrl, SecretKey};
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
    let url = figure_out_relay()
        .await?
        .ok_or_else(|| anyhow::anyhow!("missing preferred relay"))?;
    let _client = ClientBuilder::new(url, secret_key, ()).connect().await?;

    tracing::info!("Connected :)");

    Ok(())
}

pub async fn figure_out_relay() -> Result<Option<RelayUrl>> {
    let mut client = iroh_net_report::Client::new()?;
    let report = client.get_report(relay_map::default_relay_map()).await?;
    tracing::info!("Finished report: {report:#?}");
    Ok(report.preferred_relay.clone())
}

mod relay_map {
    use iroh_relay::{RelayMap, RelayNode, RelayQuicConfig};
    use url::Url;

    /// Hostname of the default NA relay.
    pub const NA_RELAY_HOSTNAME: &str = "use1-1.relay.iroh.network.";
    /// Hostname of the default EU relay.
    pub const EU_RELAY_HOSTNAME: &str = "euw1-1.relay.iroh.network.";
    /// Hostname of the default Asia-Pacific relay.
    pub const AP_RELAY_HOSTNAME: &str = "aps1-1.relay.iroh.network.";

    pub const DEFAULT_STUN_PORT: u16 = 3478;

    /// Get the default [`RelayMap`].
    pub fn default_relay_map() -> RelayMap {
        RelayMap::from_nodes([
            default_na_relay_node(),
            default_eu_relay_node(),
            default_ap_relay_node(),
            RelayNode {
                url: "https://philipp.iroh.link.".parse().unwrap(),
                stun_only: false,
                stun_port: DEFAULT_STUN_PORT,
                quic: None,
            },
        ])
        .expect("default nodes invalid")
    }

    /// Get the default [`RelayNode`] for NA.
    pub fn default_na_relay_node() -> RelayNode {
        // The default NA relay server run by number0.
        let url: Url = format!("https://{NA_RELAY_HOSTNAME}")
            .parse()
            .expect("default url");
        RelayNode {
            url: url.into(),
            stun_only: false,
            stun_port: DEFAULT_STUN_PORT,
            quic: Some(RelayQuicConfig::default()),
        }
    }

    /// Get the default [`RelayNode`] for EU.
    pub fn default_eu_relay_node() -> RelayNode {
        // The default EU relay server run by number0.
        let url: Url = format!("https://{EU_RELAY_HOSTNAME}")
            .parse()
            .expect("default_url");
        RelayNode {
            url: url.into(),
            stun_only: false,
            stun_port: DEFAULT_STUN_PORT,
            quic: Some(RelayQuicConfig::default()),
        }
    }

    /// Get the default [`RelayNode`] for Asia-Pacific
    pub fn default_ap_relay_node() -> RelayNode {
        // The default Asia-Pacific relay server run by number0.
        let url: Url = format!("https://{AP_RELAY_HOSTNAME}")
            .parse()
            .expect("default_url");
        RelayNode {
            url: url.into(),
            stun_only: false,
            stun_port: DEFAULT_STUN_PORT,
            quic: Some(RelayQuicConfig::default()),
        }
    }
}
