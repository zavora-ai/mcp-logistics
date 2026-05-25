//! Google Maps / HERE route optimization backend.
use crate::types::*;
use anyhow::Result;
use reqwest::Client;

#[derive(Clone)]
pub struct GoogleMapsBackend { http: Client, key: String }

impl GoogleMapsBackend {
    pub fn new(key: String) -> Self { Self { http: Client::new(), key } }
}

#[async_trait::async_trait]
impl RouteBackend for GoogleMapsBackend {
    fn name(&self) -> &str { "google_maps" }

    async fn optimize_route(&self, stops: &[String]) -> Result<RouteResult> {
        if stops.len() < 2 { anyhow::bail!("Need at least 2 stops"); }
        let origin = &stops[0];
        let dest = stops.last().unwrap();
        let waypoints = if stops.len() > 2 { format!("optimize:true|{}", stops[1..stops.len()-1].join("|")) } else { String::new() };
        let mut url = format!("https://maps.googleapis.com/maps/api/directions/json?origin={}&destination={}&key={}", urlenc(origin), urlenc(dest), self.key);
        if !waypoints.is_empty() { url.push_str(&format!("&waypoints={}", urlenc(&waypoints))); }
        let resp: serde_json::Value = self.http.get(&url).send().await?.error_for_status()?.json().await?;
        let route = &resp["routes"][0];
        let empty = vec![];
        let legs = route["legs"].as_array().unwrap_or(&empty);
        let total_distance_km = legs.iter().map(|l| l["distance"]["value"].as_f64().unwrap_or(0.0)).sum::<f64>() / 1000.0;
        let total_duration_min = legs.iter().map(|l| l["duration"]["value"].as_f64().unwrap_or(0.0)).sum::<f64>() / 60.0;
        let order = route["waypoint_order"].as_array().map(|a| a.iter().filter_map(|v| v.as_u64()).map(|i| i as usize).collect::<Vec<_>>()).unwrap_or_default();
        let mut result_stops = Vec::new();
        for (i, stop) in stops.iter().enumerate() {
            let actual_order = if i == 0 { 0 } else if i == stops.len() - 1 { stops.len() as u32 - 1 } else { order.get(i - 1).map(|&o| o as u32 + 1).unwrap_or(i as u32) };
            result_stops.push(RouteStop { address: stop.clone(), order: actual_order, eta: None });
        }
        result_stops.sort_by_key(|s| s.order);
        Ok(RouteResult { total_distance_km, total_duration_min, stops: result_stops, backend: "google_maps".into() })
    }

    async fn get_eta(&self, origin: &str, destination: &str) -> Result<DistanceResult> {
        self.get_distance(origin, destination).await
    }

    async fn get_distance(&self, origin: &str, destination: &str) -> Result<DistanceResult> {
        let url = format!("https://maps.googleapis.com/maps/api/distancematrix/json?origins={}&destinations={}&key={}", urlenc(origin), urlenc(destination), self.key);
        let resp: serde_json::Value = self.http.get(&url).send().await?.error_for_status()?.json().await?;
        let element = &resp["rows"][0]["elements"][0];
        if element["status"].as_str() != Some("OK") { anyhow::bail!("Route not found"); }
        Ok(DistanceResult { distance_km: element["distance"]["value"].as_f64().unwrap_or(0.0) / 1000.0, duration_min: element["duration"]["value"].as_f64().unwrap_or(0.0) / 60.0, origin: origin.into(), destination: destination.into(), backend: "google_maps".into() })
    }
}

fn urlenc(s: &str) -> String {
    s.bytes().map(|b| match b { b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(), b' ' => "+".to_string(), _ => format!("%{:02X}", b) }).collect()
}
