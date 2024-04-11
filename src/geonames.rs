use std::fmt::format;

use crate::{common::get_geonames_username, models::{build_pois, build_wiki_summaries, Geo, PlaceOfInterest, WeatherReport, WikipediaSummary}};
use serde_json::*;

pub const GEONAMES_BASE_URI: &'static str = "http://api.geonames.org";

#[derive(Debug, Copy, Clone)]
pub enum GeoNamesService {
  Postcode,
  Extended,
  Weather,
  PlacesOfInterets,
  Wikipedia,
  Address
}

impl GeoNamesService {
  pub fn to_method_name(&self) -> String {
    match self {
      Self::Postcode => "findNearbyPostalCodesJSON",
      Self::Extended => "extendedFindNearbyJSON",
      Self::Weather => "findNearByWeatherJSON",
      Self::PlacesOfInterets => "findNearbyPOIsOSMJSON",
      Self::Wikipedia => "findNearbyWikipediaJSON",
      Self::Address => "addressJSON",
      _ => ""
    }.to_string()
  }
}


async fn fetch_from_geonames(geo: Geo, service: GeoNamesService) -> Option<Map<String, Value>> {
  let req_client = reqwest::Client::new();
  let username = get_geonames_username();
  let mut query_params = vec![
    ("username", username),
    ("lat", geo.lat.to_string()),
    ("lng", geo.lng.to_string()),
  ];
  match service {
    GeoNamesService::PlacesOfInterets => {
      query_params.push(("radius", "1".to_string()));
      query_params.push(("style", "full".to_string()));
    },
    _ => {

    }
  };
  let uri = format!("{}/{}", GEONAMES_BASE_URI, service.to_method_name());
  let result = req_client.get(&uri)
    .query(&query_params).send()
    .await
    .expect("failed to get response")
    .text()
    .await;
  if let Ok(result_string) = result {
    if let Ok(json) = serde_json::from_str(&result_string) {
      Some(json)
    } else {
      None
    }
  } else {
    None
  }
}

pub async fn fetch_weather(geo: Geo) -> Option<WeatherReport> {
  if let Some(data) =  fetch_from_geonames(geo, GeoNamesService::Weather).await {
    if let Some(inner) = data.get("weatherObservation") {
      if let Some(inner_map) = inner.as_object() {
        return Some(WeatherReport::new(inner_map.to_owned()));
      }
    }
  }
  None
}

pub async fn fetch_poi(geo: Geo) -> Option<Vec<PlaceOfInterest>> {
  if let Some(data) =  fetch_from_geonames(geo, GeoNamesService::PlacesOfInterets).await {
    return Some(build_pois(data));
  }
  None
}

pub async fn fetch_wiki_entries(geo: Geo) -> Option<Vec<WikipediaSummary>> {
  if let Some(data) =  fetch_from_geonames(geo, GeoNamesService::Wikipedia).await {
    return Some(build_wiki_summaries(data));
  }
  None
}