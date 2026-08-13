//! MCP tool router for logistics operations.
use adk_mcp_sdk::{HealthCheck, HealthStatus};
use crate::types::*;
use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateShipmentInput { pub origin: Address, pub destination: Address, pub parcels: Vec<Parcel> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListShipmentsInput { #[serde(default)] pub status: Option<String>, #[serde(default = "d20")] pub limit: u32 }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct IdInput { pub id: String }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetRatesInput { pub origin: Address, pub destination: Address, pub parcels: Vec<Parcel> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CompareRatesInput { pub origin: Address, pub destination: Address, pub parcels: Vec<Parcel>, #[serde(default = "d_price")] pub sort_by: String }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AddressInput { pub address: Address }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct OptimizeRouteInput { pub stops: Vec<String> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TwoPointInput { pub origin: String, pub destination: String }

fn d20() -> u32 { 20 }
fn d_price() -> String { "price".into() }

#[derive(Clone)]
pub struct LogisticsServer {
    pub shipping: Arc<dyn ShippingBackend>,
    pub routes: Option<Arc<dyn RouteBackend>>,
}

#[tool_router]
impl LogisticsServer {
    #[tool(description = "Create a new shipment with origin, destination, and parcels")]
    async fn create_shipment(&self, Parameters(i): Parameters<CreateShipmentInput>) -> String {
        match self.shipping.create_shipment(&i.origin, &i.destination, &i.parcels).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }
    #[tool(description = "List shipments with optional status filter")]
    async fn list_shipments(&self, Parameters(i): Parameters<ListShipmentsInput>) -> String {
        match self.shipping.list_shipments(i.status.as_deref(), i.limit).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }
    #[tool(description = "Get shipment details by ID")]
    async fn get_shipment(&self, Parameters(i): Parameters<IdInput>) -> String {
        match self.shipping.get_shipment(&i.id).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }
    #[tool(description = "Cancel a shipment")]
    async fn cancel_shipment(&self, Parameters(i): Parameters<IdInput>) -> String {
        match self.shipping.cancel_shipment(&i.id).await { Ok(()) => "Shipment cancelled".into(), Err(e) => format!("Error: {e}") }
    }
    #[tool(description = "Get shipping rates for a shipment from multiple carriers")]
    async fn get_rates(&self, Parameters(i): Parameters<GetRatesInput>) -> String {
        match self.shipping.get_rates(&i.origin, &i.destination, &i.parcels).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }
    #[tool(description = "Compare rates across carriers sorted by price or speed")]
    async fn compare_rates(&self, Parameters(i): Parameters<CompareRatesInput>) -> String {
        match self.shipping.get_rates(&i.origin, &i.destination, &i.parcels).await {
            Ok(mut rates) => {
                if i.sort_by == "speed" { rates.sort_by(|a, b| a.estimated_days.cmp(&b.estimated_days)); }
                else { rates.sort_by(|a, b| a.amount.partial_cmp(&b.amount).unwrap_or(std::cmp::Ordering::Equal)); }
                serde_json::to_string_pretty(&rates).unwrap()
            }, Err(e) => format!("Error: {e}")
        }
    }
    #[tool(description = "List available carriers")]
    async fn list_carriers(&self) -> String {
        match self.shipping.list_carriers().await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }
    #[tool(description = "Book a specific rate and purchase a shipping label")]
    async fn book_rate(&self, Parameters(i): Parameters<IdInput>) -> String {
        match self.shipping.book_rate(&i.id).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }
    #[tool(description = "Get real-time tracking status for a shipment")]
    async fn track_shipment(&self, Parameters(i): Parameters<IdInput>) -> String {
        match self.shipping.track_shipment(&i.id).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }
    #[tool(description = "Get full tracking event history for a shipment")]
    async fn get_tracking_events(&self, Parameters(i): Parameters<IdInput>) -> String {
        match self.shipping.get_tracking_events(&i.id).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }
    #[tool(description = "Generate a shipping label for a booked shipment")]
    async fn generate_label(&self, Parameters(i): Parameters<IdInput>) -> String {
        match self.shipping.generate_label(&i.id).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }
    #[tool(description = "Get an existing label by shipment ID")]
    async fn get_label(&self, Parameters(i): Parameters<IdInput>) -> String {
        match self.shipping.get_label(&i.id).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }
    #[tool(description = "Validate and normalize a shipping address")]
    async fn validate_address(&self, Parameters(i): Parameters<AddressInput>) -> String {
        match self.shipping.validate_address(&i.address).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }
    #[tool(description = "Optimize delivery route for multiple stops")]
    async fn optimize_route(&self, Parameters(i): Parameters<OptimizeRouteInput>) -> String {
        match &self.routes { Some(r) => match r.optimize_route(&i.stops).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }, None => "Route backend not configured".into() }
    }
    #[tool(description = "Get estimated time of arrival between two points")]
    async fn get_eta(&self, Parameters(i): Parameters<TwoPointInput>) -> String {
        match &self.routes { Some(r) => match r.get_eta(&i.origin, &i.destination).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }, None => "Route backend not configured".into() }
    }
    #[tool(description = "Get distance and duration between two addresses")]
    async fn get_distance(&self, Parameters(i): Parameters<TwoPointInput>) -> String {
        match &self.routes { Some(r) => match r.get_distance(&i.origin, &i.destination).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }, None => "Route backend not configured".into() }
    }
}

#[async_trait::async_trait]
impl HealthCheck for LogisticsServer {
    async fn check_health(&self) -> HealthStatus {
        match self.shipping.list_carriers().await {
            Ok(_) => HealthStatus { healthy: true, message: Some(format!("{} connected", self.shipping.name())), latency_ms: Some(1) },
            Err(e) => HealthStatus { healthy: false, message: Some(format!("{}: {e}", self.shipping.name())), latency_ms: None },
        }
    }
}

adk_mcp_sdk::mcp_2026_server! {
    server: LogisticsServer,
    task_tools: ["generate_label", "optimize_route"],
    approval_tools: ["create_shipment", "cancel_shipment", "book_rate"],
    cache_ttl_ms: 60_000,
}
