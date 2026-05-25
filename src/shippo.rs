//! Shippo API backend — multi-carrier shipping.
use crate::types::*;
use anyhow::Result;
use reqwest::Client;

const BASE: &str = "https://api.goshippo.com";

#[derive(Clone)]
pub struct ShippoBackend { http: Client, token: String }

impl ShippoBackend {
    pub fn new(token: String) -> Self { Self { http: Client::new(), token } }
    async fn get(&self, path: &str) -> Result<serde_json::Value> {
        Ok(self.http.get(format!("{BASE}/{path}")).header("Authorization", format!("ShippoToken {}", self.token)).send().await?.error_for_status()?.json().await?)
    }
    async fn post(&self, path: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
        Ok(self.http.post(format!("{BASE}/{path}")).header("Authorization", format!("ShippoToken {}", self.token)).json(body).send().await?.error_for_status()?.json().await?)
    }
    fn to_addr(a: &Address) -> serde_json::Value {
        serde_json::json!({"name": a.name.as_deref().unwrap_or(""), "street1": a.street1, "street2": a.street2, "city": a.city, "state": a.state, "zip": a.postal_code, "country": a.country, "phone": a.phone})
    }
}

#[async_trait::async_trait]
impl ShippingBackend for ShippoBackend {
    fn name(&self) -> &str { "shippo" }

    async fn create_shipment(&self, origin: &Address, destination: &Address, parcels: &[Parcel]) -> Result<Shipment> {
        let pkgs: Vec<_> = parcels.iter().map(|p| serde_json::json!({"mass_unit": "kg", "weight": p.weight_kg.to_string(), "length": p.length_cm.unwrap_or(10.0).to_string(), "width": p.width_cm.unwrap_or(10.0).to_string(), "height": p.height_cm.unwrap_or(10.0).to_string(), "distance_unit": "cm"})).collect();
        let resp = self.post("shipments", &serde_json::json!({"address_from": Self::to_addr(origin), "address_to": Self::to_addr(destination), "parcels": pkgs, "async": false})).await?;
        Ok(Shipment { id: resp["object_id"].as_str().unwrap_or("").into(), status: resp["status"].as_str().unwrap_or("QUEUED").into(), origin: origin.clone(), destination: destination.clone(), parcels: parcels.to_vec(), carrier: None, tracking_number: None, label_url: None, created_at: resp["object_created"].as_str().map(Into::into), backend: "shippo".into() })
    }

    async fn list_shipments(&self, _status: Option<&str>, limit: u32) -> Result<Vec<Shipment>> {
        let resp = self.get(&format!("shipments?results={limit}")).await?;
        Ok(resp["results"].as_array().map(|a| a.iter().map(|s| Shipment { id: s["object_id"].as_str().unwrap_or("").into(), status: s["status"].as_str().unwrap_or("").into(), origin: Address { name: None, street1: String::new(), street2: None, city: String::new(), state: None, postal_code: None, country: String::new(), phone: None }, destination: Address { name: None, street1: String::new(), street2: None, city: String::new(), state: None, postal_code: None, country: String::new(), phone: None }, parcels: vec![], carrier: None, tracking_number: None, label_url: None, created_at: s["object_created"].as_str().map(Into::into), backend: "shippo".into() }).collect()).unwrap_or_default())
    }

    async fn get_shipment(&self, id: &str) -> Result<Shipment> {
        let s = self.get(&format!("shipments/{id}")).await?;
        Ok(Shipment { id: s["object_id"].as_str().unwrap_or("").into(), status: s["status"].as_str().unwrap_or("").into(), origin: Address { name: None, street1: String::new(), street2: None, city: String::new(), state: None, postal_code: None, country: String::new(), phone: None }, destination: Address { name: None, street1: String::new(), street2: None, city: String::new(), state: None, postal_code: None, country: String::new(), phone: None }, parcels: vec![], carrier: None, tracking_number: None, label_url: None, created_at: s["object_created"].as_str().map(Into::into), backend: "shippo".into() })
    }

    async fn cancel_shipment(&self, _id: &str) -> Result<()> { anyhow::bail!("Shippo: cancel via refund on the transaction") }

    async fn get_rates(&self, origin: &Address, destination: &Address, parcels: &[Parcel]) -> Result<Vec<Rate>> {
        let pkgs: Vec<_> = parcels.iter().map(|p| serde_json::json!({"mass_unit": "kg", "weight": p.weight_kg.to_string(), "length": p.length_cm.unwrap_or(10.0).to_string(), "width": p.width_cm.unwrap_or(10.0).to_string(), "height": p.height_cm.unwrap_or(10.0).to_string(), "distance_unit": "cm"})).collect();
        let resp = self.post("shipments", &serde_json::json!({"address_from": Self::to_addr(origin), "address_to": Self::to_addr(destination), "parcels": pkgs, "async": false})).await?;
        Ok(resp["rates"].as_array().map(|a| a.iter().map(|r| Rate { id: r["object_id"].as_str().unwrap_or("").into(), carrier: r["provider"].as_str().unwrap_or("").into(), service: r["servicelevel"]["name"].as_str().unwrap_or("").into(), amount: r["amount"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0), currency: r["currency"].as_str().unwrap_or("USD").into(), estimated_days: r["estimated_days"].as_u64().map(|d| d as u32), backend: "shippo".into() }).collect()).unwrap_or_default())
    }

    async fn list_carriers(&self) -> Result<Vec<Carrier>> {
        let resp = self.get("carrier_accounts").await?;
        Ok(resp["results"].as_array().map(|a| a.iter().map(|c| Carrier { id: c["object_id"].as_str().unwrap_or("").into(), name: c["carrier"].as_str().unwrap_or("").into(), services: vec![], backend: "shippo".into() }).collect()).unwrap_or_default())
    }

    async fn book_rate(&self, rate_id: &str) -> Result<Shipment> {
        let resp = self.post("transactions", &serde_json::json!({"rate": rate_id, "async": false})).await?;
        Ok(Shipment { id: resp["object_id"].as_str().unwrap_or("").into(), status: "purchased".into(), origin: Address { name: None, street1: String::new(), street2: None, city: String::new(), state: None, postal_code: None, country: String::new(), phone: None }, destination: Address { name: None, street1: String::new(), street2: None, city: String::new(), state: None, postal_code: None, country: String::new(), phone: None }, parcels: vec![], carrier: resp["rate"]["provider"].as_str().map(Into::into), tracking_number: resp["tracking_number"].as_str().map(Into::into), label_url: resp["label_url"].as_str().map(Into::into), created_at: None, backend: "shippo".into() })
    }

    async fn track_shipment(&self, id: &str) -> Result<TrackingInfo> {
        let resp = self.get(&format!("tracks/{id}")).await?;
        Ok(TrackingInfo { tracking_number: resp["tracking_number"].as_str().unwrap_or("").into(), status: resp["tracking_status"]["status"].as_str().unwrap_or("").into(), carrier: resp["carrier"].as_str().map(Into::into), estimated_delivery: resp["eta"].as_str().map(Into::into), events: resp["tracking_history"].as_array().map(|a| a.iter().map(|e| TrackingEvent { timestamp: e["status_date"].as_str().unwrap_or("").into(), status: e["status"].as_str().unwrap_or("").into(), location: e["location"]["city"].as_str().map(Into::into), description: e["status_details"].as_str().unwrap_or("").into() }).collect()).unwrap_or_default(), backend: "shippo".into() })
    }

    async fn get_tracking_events(&self, id: &str) -> Result<Vec<TrackingEvent>> { Ok(self.track_shipment(id).await?.events) }
    async fn generate_label(&self, shipment_id: &str) -> Result<Label> { let s = self.get_shipment(shipment_id).await?; Ok(Label { shipment_id: shipment_id.into(), tracking_number: s.tracking_number.unwrap_or_default(), label_url: s.label_url.unwrap_or_default(), format: "pdf".into(), backend: "shippo".into() }) }
    async fn get_label(&self, shipment_id: &str) -> Result<Label> { self.generate_label(shipment_id).await }
    async fn validate_address(&self, address: &Address) -> Result<Address> {
        let resp = self.post("addresses", &Self::to_addr(address)).await?;
        if resp["validation_results"]["is_valid"].as_bool() == Some(true) { Ok(address.clone()) } else { anyhow::bail!("Address validation failed") }
    }
}
