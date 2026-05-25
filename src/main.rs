//! mcp-logistics — Enterprise Logistics MCP Server
mod types;
mod server;

#[cfg(feature = "shipengine")]
mod shipengine;
#[cfg(feature = "shippo")]
mod shippo;
#[cfg(feature = "sendy")]
mod sendy;
#[cfg(feature = "routes")]
mod routes;

use rmcp::{ServiceExt, transport::stdio};
use server::LogisticsServer;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let manifest = adk_mcp_sdk::ServerManifest::from_file(std::path::Path::new("mcp-server.toml"))?;
    let errors = manifest.validate();
    if !errors.is_empty() {
        for e in &errors { tracing::error!("manifest: {e}"); }
        anyhow::bail!("invalid mcp-server.toml ({} error(s))", errors.len());
    }

    let shipping: Arc<dyn types::ShippingBackend> = init_shipping()?;
    let route_backend: Option<Arc<dyn types::RouteBackend>> = init_routes();

    tracing::info!("{} v{} starting on stdio (shipping: {}{})", manifest.display_name, manifest.version, shipping.name(), route_backend.as_ref().map(|r| format!(", routes: {}", r.name())).unwrap_or_default());
    let server = LogisticsServer { shipping, routes: route_backend };
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

fn init_shipping() -> anyhow::Result<Arc<dyn types::ShippingBackend>> {
    #[cfg(feature = "shipengine")]
    if let Ok(key) = std::env::var("SHIPENGINE_API_KEY") {
        tracing::info!("Using ShipEngine backend");
        return Ok(Arc::new(shipengine::ShipEngineBackend::new(key)));
    }

    #[cfg(feature = "shippo")]
    if let Ok(token) = std::env::var("SHIPPO_TOKEN") {
        tracing::info!("Using Shippo backend");
        return Ok(Arc::new(shippo::ShippoBackend::new(token)));
    }

    #[cfg(feature = "sendy")]
    if let Ok(key) = std::env::var("SENDY_API_KEY") {
        tracing::info!("Using Sendy (Africa) backend");
        return Ok(Arc::new(sendy::SendyBackend::new(key)));
    }

    anyhow::bail!("No shipping backend configured. Set one of: SHIPENGINE_API_KEY, SHIPPO_TOKEN, SENDY_API_KEY")
}

fn init_routes() -> Option<Arc<dyn types::RouteBackend>> {
    #[cfg(feature = "routes")]
    if let Ok(key) = std::env::var("GOOGLE_MAPS_KEY") {
        tracing::info!("Google Maps route optimization enabled");
        return Some(Arc::new(routes::GoogleMapsBackend::new(key)));
    }
    None
}
