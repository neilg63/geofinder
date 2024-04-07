use axum::{http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use serde_json::{json, Value};
use serde_with::skip_serializing_none;
use simple_string_patterns::*;

use crate::models::Geo;

pub fn get_db_name() -> String {
  dotenv::var("MONGO_DB_NAME").unwrap_or("none".to_string())
}

pub fn get_gtz_url() -> String {
  dotenv::var("GEOTIMEZONE_API").unwrap_or("http://localhost:8080".to_string())
}

pub async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}

// basic handler that responds with a static string
pub async fn welcome() -> &'static str {
    "Welcome to Multifaceted Web Services"
}

// basic handler that responds with statuc JSON
pub async fn unauthorized() -> Value {
    json!({
      "valid": false,
      "message": "Unauthorized"
    })
}

pub(crate) fn build_store_key_from_geo(prefix: &str, geo: Geo, radius: Option<f64>, limit: Option<u32>) -> String {
  let mut extras: Vec<String> = Vec::new();
  if let Some(rv) = radius {
    extras.push(format!("_{:2}", rv));
  }
  if let Some(lv) = limit {
    extras.push(format!("_{}", lv));
  }
  let extra = if extras.len() > 0 { extras.concat() } else { "".to_owned() };
  format!("{}_{:4}_{:4}{}", prefix, geo.lat,geo.lng, extra)
}

#[skip_serializing_none]
#[derive(Deserialize, Debug, Clone)]
pub struct GeoParams {
  pub loc: Option<String>,
  pub search: Option<String>,
  pub dt: Option<String>,
  pub km: Option<f64>,
  pub skip: Option<u32>,
  pub limit: Option<u32>,
  pub code: Option<String>,
}

impl GeoParams {
  pub fn to_geo_opt(&self) -> Option<Geo> {
    let nums = if let Some(loc_str) = self.loc.clone() {
      loc_str.split_to_numbers::<f64>(",")
    } else {
      vec![]
    };
    if nums.len() > 1 {
      let lat = nums.get(0).unwrap_or(&0.0).to_owned();
      let lng = nums.get(1).unwrap_or(&0.0).to_owned();
      if nums.len() == 2 {
        Some(Geo::simple(lat, lng))
      } else {
        let alt = nums.get(2).unwrap_or(&0.0).to_owned();
        Some(Geo::new(lat, lng, alt))
      }
    } else {
      None
    }
  }
}