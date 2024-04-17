use std::fs::read_to_string;
use axum::{http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use serde_json::{json, Value};
use serde_with::skip_serializing_none;
use simple_string_patterns::*;
use string_patterns::PatternMatch;
use crate::models::Geo;

pub fn get_db_name() -> String {
  dotenv::var("MONGO_DB_NAME").unwrap_or("none".to_string())
}

pub fn get_gtz_url() -> String {
  dotenv::var("GEOTIMEZONE_API").unwrap_or("http://localhost:8080".to_string())
}

pub fn get_geonames_username() -> String {
  dotenv::var("GEONAMES_USERNAME").unwrap_or("demo".to_string())
}

pub fn get_addresses_url() -> String {
  dotenv::var("ADDRESSES_API").unwrap_or("http://localhost:8080".to_string())
}

pub fn get_astro_url() -> String {
  dotenv::var("ASTRO_API").unwrap_or("http://localhost:8080".to_string())
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
    extras.push(format!("_{:2}", rv).strip_by_type(CharType::Spaces));
  }
  if let Some(lv) = limit {
    extras.push(format!("_{}", lv).strip_by_type(CharType::Spaces));
  }
  let extra = if extras.len() > 0 { extras.concat().trim().to_owned() } else { "".to_owned() };
  let g_c = format!("{:5}_{:5}",geo.lat,geo.lng);
  format!("{}_{}{}", prefix, g_c, extra)
}

#[skip_serializing_none]
#[derive(Deserialize, Debug, Clone)]
pub struct GeoParams {
  pub loc: Option<String>,
  pub search: Option<String>,
  pub place: Option<String>,
  pub dt: Option<String>,
  pub km: Option<f64>,
  pub skip: Option<u32>,
  pub limit: Option<u32>,
  pub code: Option<String>,
  pub fuzzy: Option<u32>,
  pub cc: Option<String>,
  pub zn: Option<String>,
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

#[skip_serializing_none]
#[derive(Deserialize, Debug, Clone)]
pub struct PostParams {
  pub lat: Option<f64>,
  pub lng: Option<f64>,
  pub pc: Option<String>,
  pub km: Option<f64>,
  pub skip: Option<u32>,
  pub limit: Option<u32>,
  pub code: Option<String>,
}

pub fn is_valid_date_string(dt_str: &str) -> bool {
  dt_str.pattern_match_cs(r#"^\d\d\d\d-[01]\d-[0-3]\d"#)
}

pub fn is_valid_zone_name(dt_str: &str) -> bool {
  dt_str.pattern_match_cs(r#"^\w+/\w+"#)
}

pub fn is_valid_uk_postcode(dt_str: &str) -> bool {
  dt_str.pattern_match_cs(r#"^[A-Z]+\d\s\d"#)
}

pub fn natural_tz_offset_from_utc(lng: f64) -> i64 {
  let lng360 = (lng + 540f64) % 360f64;
  let lng180 = lng360 - 180f64;
  (lng180 * 4f64 * 60f64) as i64
}

pub fn read_lines(filename: &str) -> Vec<String> {
  let mut result = Vec::new();
  if let Ok(file_content) = read_to_string(filename) {
    for line in file_content.lines() {
      result.push(line.to_string())
    }
  }
  result
}