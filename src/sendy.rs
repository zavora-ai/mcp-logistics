//! Sendy API backend — Africa last-mile delivery (Kenya, Nigeria, Uganda).
use crate::types::*;
use anyhow::Result;
use reqwest::Client;

const BASE: &str = "https://api.sendy.co.ke/v2";

#[derive(Clone)]
pub struct SendyBackend { http: Client, api_key: String }

impl SendyBackend {
    pub fn new(api_key: String) -> Self { Self { http: Client::new(), api_key } }
    async fn get(&self, path: &str) -> Result<serde_json::Value> {
        Ok(self.http.get(format!("{BASE}/{path}")).header("Authorization", format!("Bearer {}", self.api_key)).send().await?.error_for_status()?.json().await?)
    }
    async fn post(&self, path: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
        Ok(self.http.post(format!("{BASE}/{path}")).header("Authorization", format!("Bearer {}", self.api_key)).json(body).send().await?.error_for_status()?.json().await?)
    }
}

#[async_trait::async_trait]
impl ShippingBackend for SendyBackend {
    fn name(&self) -> &str { "sendy" }

    async fn create_shipment(&self, origin: &Address, destination: &Address, parcels: &[Parcel]) -> Result<Shipment> {
        let weight = parcels.iter().map(|p| p.weight_kg).sum::<f64>();
        let body = serde_json::json!({"pickup": {"name": origin.name, "address": origin.street1, "city": origin.city, "country": origin.country, "phone": origin.phone}, "delivery": {"name": destination.name, "address": destination.street1, "city": destination.city, "country": destination.country, "phone": destination.phone}, "package": {"weight": weight, "description": parcels.first().and_then(|p| p.description.as_deref()).unwrap_or("Package")}});
        let resp = self.post("orders", &body).await?;
        Ok(Shipment { id: resp["order_id"].as_str().unwrap_or("").into(), status: resp["status"].as_str().unwrap_or("pending").into(), origin: origin.clone(), destination: destination.clone(), parcels: parcels.to_vec(), carrier: Some("Sendy".into()), tracking_number: resp["tracking_number"].as_str().map(Into::into), label_url: None, created_at: None, backend: "sendy".into() })
    }

    async fn list_shipments(&self, _status: Option<&str>, limit: u32) -> Result<Vec<Shipment>> {
        let resp = self.get(&format!("orders?limit={limit}")).await?;
        Ok(resp["data"].as_array().map(|a| a.iter().map(|s| Shipment { id: s["order_id"].as_str().unwrap_or("").into(), status: s["status"].as_str().unwrap_or("").into(), origin: Address { name: None, street1: s["pickup"]["address"].as_str().unwrap_or("").into(), street2: None, city: s["pickup"]["city"].as_str().unwrap_or("").into(), state: None, postal_code: None, country: "KE".into(), phone: None }, destination: Address { name: None, street1: s["delivery"]["address"].as_str().unwrap_or("").into(), street2: None, city: s["delivery"]["city"].as_str().unwrap_or("").into(), state: None, postal_code: None, country: "KE".into(), phone: None }, parcels: vec![], carrier: Some("Sendy".into()), tracking_number: s["tracking_number"].as_str().map(Into::into), label_url: None, created_at: s["created_at"].as_str().map(Into::into), backend: "sendy".into() }).collect()).unwrap_or_default())
    }

    async fn get_shipment(&self, id: &str) -> Result<Shipment> {
        let s = self.get(&format!("orders/{id}")).await?;
        Ok(Shipment { id: s["order_id"].as_str().unwrap_or("").into(), status: s["status"].as_str().unwrap_or("").into(), origin: Address { name: None, street1: s["pickup"]["address"].as_str().unwrap_or("").into(), street2: None, city: s["pickup"]["city"].as_str().unwrap_or("").into(), state: None, postal_code: None, country: "KE".into(), phone: None }, destination: Address { name: None, street1: s["delivery"]["address"].as_str().unwrap_or("").into(), street2: None, city: s["delivery"]["city"].as_str().unwrap_or("").into(), state: None, postal_code: None, country: "KE".into(), phone: None }, parcels: vec![], carrier: Some("Sendy".into()), tracking_number: s["tracking_number"].as_str().map(Into::into), label_url: None, created_at: s["created_at"].as_str().map(Into::into), backend: "sendy".into() })
    }

    async fn cancel_shipment(&self, id: &str) -> Result<()> { self.post(&format!("orders/{id}/cancel"), &serde_json::json!({})).await?; Ok(()) }

    async fn get_rates(&self, origin: &Address, destination: &Address, parcels: &[Parcel]) -> Result<Vec<Rate>> {
        let weight = parcels.iter().map(|p| p.weight_kg).sum::<f64>();
        let body = serde_json::json!({"pickup": {"city": origin.city, "country": origin.country}, "delivery": {"city": destination.city, "country": destination.country}, "weight": weight});
        let resp = self.post("pricing", &body).await?;
        Ok(resp["rates"].as_array().map(|a| a.iter().map(|r| Rate { id: r["rate_id"].as_str().unwrap_or("").into(), carrier: "Sendy".into(), service: r["service_type"].as_str().unwrap_or("standard").into(), amount: r["amount"].as_f64().unwrap_or(0.0), currency: r["currency"].as_str().unwrap_or("KES").into(), estimated_days: r["estimated_days"].as_u64().map(|d| d as u32), backend: "sendy".into() }).collect()).unwrap_or_default())
    }

    async fn list_carriers(&self) -> Result<Vec<Carrier>> {
        Ok(vec![Carrier { id: "sendy".into(), name: "Sendy".into(), services: vec!["standard".into(), "express".into(), "same_day".into()], backend: "sendy".into() }])
    }

    async fn book_rate(&self, rate_id: &str) -> Result<Shipment> {
        let resp = self.post("orders/confirm", &serde_json::json!({"rate_id": rate_id})).await?;
        Ok(Shipment { id: resp["order_id"].as_str().unwrap_or("").into(), status: "confirmed".into(), origin: Address { name: None, street1: String::new(), street2: None, city: String::new(), state: None, postal_code: None, country: "KE".into(), phone: None }, destination: Address { name: None, street1: String::new(), street2: None, city: String::new(), state: None, postal_code: None, country: "KE".into(), phone: None }, parcels: vec![], carrier: Some("Sendy".into()), tracking_number: resp["tracking_number"].as_str().map(Into::into), label_url: None, created_at: None, backend: "sendy".into() })
    }

    async fn track_shipment(&self, id: &str) -> Result<TrackingInfo> {
        let resp = self.get(&format!("orders/{id}/tracking")).await?;
        Ok(TrackingInfo { tracking_number: resp["tracking_number"].as_str().unwrap_or("").into(), status: resp["status"].as_str().unwrap_or("").into(), carrier: Some("Sendy".into()), estimated_delivery: resp["eta"].as_str().map(Into::into), events: resp["events"].as_array().map(|a| a.iter().map(|e| TrackingEvent { timestamp: e["timestamp"].as_str().unwrap_or("").into(), status: e["status"].as_str().unwrap_or("").into(), location: e["location"].as_str().map(Into::into), description: e["description"].as_str().unwrap_or("").into() }).collect()).unwrap_or_default(), backend: "sendy".into() })
    }

    async fn get_tracking_events(&self, id: &str) -> Result<Vec<TrackingEvent>> { Ok(self.track_shipment(id).await?.events) }
    async fn generate_label(&self, _id: &str) -> Result<Label> { anyhow::bail!("Sendy does not generate printable labels — riders use the app") }
    async fn get_label(&self, _id: &str) -> Result<Label> { anyhow::bail!("Sendy does not generate printable labels") }
    async fn validate_address(&self, address: &Address) -> Result<Address> { Ok(address.clone()) }
}
