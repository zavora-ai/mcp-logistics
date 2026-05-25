//! Unified logistics types.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Address {
    pub name: Option<String>,
    pub street1: String,
    pub street2: Option<String>,
    pub city: String,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: String,
    pub phone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Parcel {
    pub weight_kg: f64,
    pub length_cm: Option<f64>,
    pub width_cm: Option<f64>,
    pub height_cm: Option<f64>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shipment {
    pub id: String,
    pub status: String,
    pub origin: Address,
    pub destination: Address,
    pub parcels: Vec<Parcel>,
    pub carrier: Option<String>,
    pub tracking_number: Option<String>,
    pub label_url: Option<String>,
    pub created_at: Option<String>,
    pub backend: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rate {
    pub id: String,
    pub carrier: String,
    pub service: String,
    pub amount: f64,
    pub currency: String,
    pub estimated_days: Option<u32>,
    pub backend: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Carrier {
    pub id: String,
    pub name: String,
    pub services: Vec<String>,
    pub backend: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingEvent {
    pub timestamp: String,
    pub status: String,
    pub location: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingInfo {
    pub tracking_number: String,
    pub status: String,
    pub carrier: Option<String>,
    pub estimated_delivery: Option<String>,
    pub events: Vec<TrackingEvent>,
    pub backend: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub shipment_id: String,
    pub tracking_number: String,
    pub label_url: String,
    pub format: String,
    pub backend: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteStop {
    pub address: String,
    pub order: u32,
    pub eta: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteResult {
    pub total_distance_km: f64,
    pub total_duration_min: f64,
    pub stops: Vec<RouteStop>,
    pub backend: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistanceResult {
    pub distance_km: f64,
    pub duration_min: f64,
    pub origin: String,
    pub destination: String,
    pub backend: String,
}

/// Shipping backend trait.
#[async_trait::async_trait]
pub trait ShippingBackend: Send + Sync {
    fn name(&self) -> &str;
    async fn create_shipment(&self, origin: &Address, destination: &Address, parcels: &[Parcel]) -> anyhow::Result<Shipment>;
    async fn list_shipments(&self, status: Option<&str>, limit: u32) -> anyhow::Result<Vec<Shipment>>;
    async fn get_shipment(&self, id: &str) -> anyhow::Result<Shipment>;
    async fn cancel_shipment(&self, id: &str) -> anyhow::Result<()>;
    async fn get_rates(&self, origin: &Address, destination: &Address, parcels: &[Parcel]) -> anyhow::Result<Vec<Rate>>;
    async fn list_carriers(&self) -> anyhow::Result<Vec<Carrier>>;
    async fn book_rate(&self, rate_id: &str) -> anyhow::Result<Shipment>;
    async fn track_shipment(&self, shipment_id: &str) -> anyhow::Result<TrackingInfo>;
    async fn get_tracking_events(&self, shipment_id: &str) -> anyhow::Result<Vec<TrackingEvent>>;
    async fn generate_label(&self, shipment_id: &str) -> anyhow::Result<Label>;
    async fn get_label(&self, shipment_id: &str) -> anyhow::Result<Label>;
    async fn validate_address(&self, address: &Address) -> anyhow::Result<Address>;
}

/// Route optimization backend trait.
#[async_trait::async_trait]
pub trait RouteBackend: Send + Sync {
    fn name(&self) -> &str;
    async fn optimize_route(&self, stops: &[String]) -> anyhow::Result<RouteResult>;
    async fn get_eta(&self, origin: &str, destination: &str) -> anyhow::Result<DistanceResult>;
    async fn get_distance(&self, origin: &str, destination: &str) -> anyhow::Result<DistanceResult>;
}
