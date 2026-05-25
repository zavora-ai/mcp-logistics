//! ShipEngine API backend — multi-carrier (FedEx, UPS, DHL, USPS).
use crate::types::*;
use anyhow::Result;
use reqwest::Client;

const BASE: &str = "https://api.shipengine.com/v1";

#[derive(Clone)]
pub struct ShipEngineBackend { http: Client, key: String }

impl ShipEngineBackend {
    pub fn new(key: String) -> Self { Self { http: Client::new(), key } }
    async fn get(&self, path: &str) -> Result<serde_json::Value> {
        Ok(self.http.get(format!("{BASE}/{path}")).header("API-Key", &self.key).send().await?.error_for_status()?.json().await?)
    }
    async fn post(&self, path: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
        Ok(self.http.post(format!("{BASE}/{path}")).header("API-Key", &self.key).json(body).send().await?.error_for_status()?.json().await?)
    }
    async fn put(&self, path: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
        Ok(self.http.put(format!("{BASE}/{path}")).header("API-Key", &self.key).json(body).send().await?.error_for_status()?.json().await?)
    }
    #[allow(dead_code)]
    async fn delete(&self, path: &str) -> Result<()> {
        self.http.delete(format!("{BASE}/{path}")).header("API-Key", &self.key).send().await?.error_for_status()?; Ok(())
    }
    fn to_se_address(a: &Address) -> serde_json::Value {
        serde_json::json!({"name": a.name, "address_line1": a.street1, "address_line2": a.street2, "city_locality": a.city, "state_province": a.state, "postal_code": a.postal_code, "country_code": a.country, "phone": a.phone})
    }
    fn from_se_address(v: &serde_json::Value) -> Address {
        Address { name: v["name"].as_str().map(Into::into), street1: v["address_line1"].as_str().unwrap_or("").into(), street2: v["address_line2"].as_str().map(Into::into), city: v["city_locality"].as_str().unwrap_or("").into(), state: v["state_province"].as_str().map(Into::into), postal_code: v["postal_code"].as_str().map(Into::into), country: v["country_code"].as_str().unwrap_or("").into(), phone: v["phone"].as_str().map(Into::into) }
    }
}

#[async_trait::async_trait]
impl ShippingBackend for ShipEngineBackend {
    fn name(&self) -> &str { "shipengine" }

    async fn create_shipment(&self, origin: &Address, destination: &Address, parcels: &[Parcel]) -> Result<Shipment> {
        let pkgs: Vec<_> = parcels.iter().map(|p| serde_json::json!({"weight": {"value": p.weight_kg, "unit": "kilogram"}, "dimensions": {"length": p.length_cm.unwrap_or(10.0), "width": p.width_cm.unwrap_or(10.0), "height": p.height_cm.unwrap_or(10.0), "unit": "centimeter"}})).collect();
        let body = serde_json::json!({"shipments": [{"ship_from": Self::to_se_address(origin), "ship_to": Self::to_se_address(destination), "packages": pkgs}]});
        let resp = self.post("shipments", &body).await?;
        let s = &resp["shipments"][0];
        Ok(Shipment { id: s["shipment_id"].as_str().unwrap_or("").into(), status: s["shipment_status"].as_str().unwrap_or("pending").into(), origin: origin.clone(), destination: destination.clone(), parcels: parcels.to_vec(), carrier: None, tracking_number: None, label_url: None, created_at: s["created_at"].as_str().map(Into::into), backend: "shipengine".into() })
    }

    async fn list_shipments(&self, status: Option<&str>, limit: u32) -> Result<Vec<Shipment>> {
        let mut path = format!("shipments?page_size={limit}");
        if let Some(s) = status { path.push_str(&format!("&shipment_status={s}")); }
        let resp = self.get(&path).await?;
        Ok(resp["shipments"].as_array().map(|a| a.iter().map(|s| Shipment { id: s["shipment_id"].as_str().unwrap_or("").into(), status: s["shipment_status"].as_str().unwrap_or("").into(), origin: Self::from_se_address(&s["ship_from"]), destination: Self::from_se_address(&s["ship_to"]), parcels: vec![], carrier: s["carrier_id"].as_str().map(Into::into), tracking_number: s["tracking_number"].as_str().map(Into::into), label_url: None, created_at: s["created_at"].as_str().map(Into::into), backend: "shipengine".into() }).collect()).unwrap_or_default())
    }

    async fn get_shipment(&self, id: &str) -> Result<Shipment> {
        let s = self.get(&format!("shipments/{id}")).await?;
        Ok(Shipment { id: s["shipment_id"].as_str().unwrap_or("").into(), status: s["shipment_status"].as_str().unwrap_or("").into(), origin: Self::from_se_address(&s["ship_from"]), destination: Self::from_se_address(&s["ship_to"]), parcels: vec![], carrier: s["carrier_id"].as_str().map(Into::into), tracking_number: s["tracking_number"].as_str().map(Into::into), label_url: None, created_at: s["created_at"].as_str().map(Into::into), backend: "shipengine".into() })
    }

    async fn cancel_shipment(&self, id: &str) -> Result<()> { self.put(&format!("shipments/{id}/cancel"), &serde_json::json!({})).await?; Ok(()) }

    async fn get_rates(&self, origin: &Address, destination: &Address, parcels: &[Parcel]) -> Result<Vec<Rate>> {
        let pkgs: Vec<_> = parcels.iter().map(|p| serde_json::json!({"weight": {"value": p.weight_kg, "unit": "kilogram"}, "dimensions": {"length": p.length_cm.unwrap_or(10.0), "width": p.width_cm.unwrap_or(10.0), "height": p.height_cm.unwrap_or(10.0), "unit": "centimeter"}})).collect();
        let body = serde_json::json!({"shipment": {"ship_from": Self::to_se_address(origin), "ship_to": Self::to_se_address(destination), "packages": pkgs}, "rate_options": {"carrier_ids": []}});
        let resp = self.post("rates", &body).await?;
        Ok(resp["rate_response"]["rates"].as_array().map(|a| a.iter().map(|r| Rate { id: r["rate_id"].as_str().unwrap_or("").into(), carrier: r["carrier_friendly_name"].as_str().unwrap_or("").into(), service: r["service_type"].as_str().unwrap_or("").into(), amount: r["shipping_amount"]["amount"].as_f64().unwrap_or(0.0), currency: r["shipping_amount"]["currency"].as_str().unwrap_or("USD").into(), estimated_days: r["delivery_days"].as_u64().map(|d| d as u32), backend: "shipengine".into() }).collect()).unwrap_or_default())
    }

    async fn list_carriers(&self) -> Result<Vec<Carrier>> {
        let resp = self.get("carriers").await?;
        Ok(resp["carriers"].as_array().map(|a| a.iter().map(|c| Carrier { id: c["carrier_id"].as_str().unwrap_or("").into(), name: c["friendly_name"].as_str().unwrap_or("").into(), services: c["services"].as_array().map(|s| s.iter().filter_map(|v| v["name"].as_str().map(Into::into)).collect()).unwrap_or_default(), backend: "shipengine".into() }).collect()).unwrap_or_default())
    }

    async fn book_rate(&self, rate_id: &str) -> Result<Shipment> {
        let resp = self.post("labels", &serde_json::json!({"rate_id": rate_id})).await?;
        Ok(Shipment { id: resp["shipment_id"].as_str().unwrap_or("").into(), status: "label_purchased".into(), origin: Address { name: None, street1: String::new(), street2: None, city: String::new(), state: None, postal_code: None, country: String::new(), phone: None }, destination: Address { name: None, street1: String::new(), street2: None, city: String::new(), state: None, postal_code: None, country: String::new(), phone: None }, parcels: vec![], carrier: resp["carrier_id"].as_str().map(Into::into), tracking_number: resp["tracking_number"].as_str().map(Into::into), label_url: resp["label_download"]["href"].as_str().map(Into::into), created_at: None, backend: "shipengine".into() })
    }

    async fn track_shipment(&self, id: &str) -> Result<TrackingInfo> {
        let s = self.get_shipment(id).await?;
        let tn = s.tracking_number.as_deref().unwrap_or("");
        let carrier = s.carrier.as_deref().unwrap_or("");
        let resp = self.get(&format!("tracking?carrier_code={carrier}&tracking_number={tn}")).await?;
        Ok(TrackingInfo { tracking_number: tn.into(), status: resp["status_code"].as_str().unwrap_or("").into(), carrier: Some(carrier.into()), estimated_delivery: resp["estimated_delivery_date"].as_str().map(Into::into), events: resp["events"].as_array().map(|a| a.iter().map(|e| TrackingEvent { timestamp: e["occurred_at"].as_str().unwrap_or("").into(), status: e["status_code"].as_str().unwrap_or("").into(), location: e["city_locality"].as_str().map(Into::into), description: e["description"].as_str().unwrap_or("").into() }).collect()).unwrap_or_default(), backend: "shipengine".into() })
    }

    async fn get_tracking_events(&self, id: &str) -> Result<Vec<TrackingEvent>> {
        let info = self.track_shipment(id).await?;
        Ok(info.events)
    }

    async fn generate_label(&self, shipment_id: &str) -> Result<Label> {
        let resp = self.post("labels", &serde_json::json!({"shipment_id": shipment_id})).await?;
        Ok(Label { shipment_id: shipment_id.into(), tracking_number: resp["tracking_number"].as_str().unwrap_or("").into(), label_url: resp["label_download"]["href"].as_str().unwrap_or("").into(), format: "pdf".into(), backend: "shipengine".into() })
    }

    async fn get_label(&self, shipment_id: &str) -> Result<Label> { self.generate_label(shipment_id).await }

    async fn validate_address(&self, address: &Address) -> Result<Address> {
        let resp = self.post("addresses/validate", &serde_json::json!([Self::to_se_address(address)])).await?;
        let v = &resp[0]["matched_address"];
        Ok(Self::from_se_address(v))
    }
}
