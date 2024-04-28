use crate::{common::get_geonames_username, models::{build_pois, build_postcodes, build_wiki_summaries, Geo, PcZone, PlaceOfInterest, WeatherReport, WikipediaSummary}, store::{redis_get_poi, redis_get_weather, redis_get_wiki_summaries, redis_set_poi, redis_set_weather, redis_set_wiki_summaries}};
use serde_json::*;

pub const GEONAMES_BASE_URI: &'static str = "http://api.geonames.org";

#[derive(Debug, Copy, Clone)]
pub enum GeoNamesService {
  Postcode,
  Extended,
  Weather,
  PlacesOfInterest,
  Wikipedia,
  Address
}

impl GeoNamesService {
  pub fn to_method_name(&self) -> String {
    match self {
      Self::Postcode => "findNearbyPostalCodesJSON",
      Self::Extended => "extendedFindNearbyJSON",
      Self::Weather => "findNearByWeatherJSON",
      Self::PlacesOfInterest => "findNearbyPOIsOSMJSON",
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
    GeoNamesService::PlacesOfInterest => {
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

pub async fn fetch_poi_cached(geo: Geo) -> (Option<Vec<PlaceOfInterest>>, bool) {
  let ck = format!("plofint_{}", geo.to_approx_key(3));
  let mut poi_opt:Option<Vec<PlaceOfInterest>> = None;
  let mut cached = false;
  if let Some(poi) = redis_get_poi(&ck) {
    poi_opt = Some(poi);
    cached = true;
  } else {
    if let Some(poi) = fetch_poi(geo).await {
      poi_opt = Some(poi.clone());
      redis_set_poi(&ck,&poi);
    }
  }
  (poi_opt, cached)
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

pub async fn fetch_weather_cached(geo: Geo) -> (Option<WeatherReport>, bool) {
  let mut weather_opt: Option<WeatherReport> = None;
  let mut cached = false;
  let ck = format!("weather_{}", geo.to_approx_key(1));
    if let Some(weather) = redis_get_weather(&ck) {
      weather_opt = Some(weather);
      cached = true;
    } else {
      if let Some(weather) = fetch_weather(geo).await {
        weather_opt = Some(weather.clone());
        redis_set_weather(&ck,&weather);
      }
    }
  (weather_opt, cached)
}

pub async fn fetch_poi(geo: Geo) -> Option<Vec<PlaceOfInterest>> {
  if let Some(data) =  fetch_from_geonames(geo, GeoNamesService::PlacesOfInterest).await {
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

pub async fn fetch_wiki_entries_cached(geo: Geo) -> (Option<Vec<WikipediaSummary>>, bool) {
  let mut items_opt: Option<Vec<WikipediaSummary>> = None;
  let mut cached = false;
  let ck = format!("wiki_{}", geo.to_approx_key(3));
  if let Some(stored_items) = redis_get_wiki_summaries(&ck) {
    cached = true;
    items_opt = Some(stored_items);
  } else {
    if let Some(items) = fetch_wiki_entries(geo).await {
      items_opt = Some(items.clone());
      redis_set_wiki_summaries(&ck, &items);
    }
  }
  (items_opt, cached)
}

pub async fn fetch_postcodes(geo: Geo) -> Option<Vec<PcZone>> {
  if let Some(data) =  fetch_from_geonames(geo, GeoNamesService::Postcode).await {
    return Some(build_postcodes(data));
  }
  None
}
