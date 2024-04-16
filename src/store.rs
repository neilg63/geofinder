
use redis::{Commands, Connection, RedisResult};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{models::{AstroData, GeoNearby, PcRow, PcZone, PlaceOfInterest, TzRow, WeatherReport, WikipediaSummary}, store};

pub(crate) fn redis_client() -> RedisResult<Connection> {
  let client = redis::Client::open("redis://127.0.0.1/")?;
  client.get_connection()
}

pub(crate) fn redis_get_opt_string<'a>(key: &str) -> Option<String> {
  if let Ok(mut connection) =  redis_client() {
    if let Ok(result) = connection.get(key.to_owned()) {
      return Some(result);
    }
  }
  None
}

pub(crate) fn redis_set_pc_results(key: &str, data: &Vec<PcRow>) -> bool {
  redis_set_data::<Vec<PcRow>>(key, data, 0)
}

pub fn redis_get_pc_results(key: &str) -> Vec<PcRow> {
  redis_get_data::<Vec<PcRow>>(key).unwrap_or(vec![])
}



pub(crate) fn redis_set_pc_zones(key: &str, data: &Vec<PcZone>) -> bool {
  redis_set_data::<Vec<PcZone>>(key, data, 0)
}

pub fn redis_get_pc_zones(key: &str) -> Vec<PcZone> {
  redis_get_data::<Vec<PcZone>>(key).unwrap_or(vec![])
}


pub fn  redis_set_geo_nearby(key: &str, data: &GeoNearby) -> bool {
  redis_set_data::<GeoNearby>(key, data, 0)
}

pub fn redis_get_geo_nearby(key: &str) -> Option<GeoNearby> {
  redis_get_data::<GeoNearby>(key)
}

pub fn redis_set_timezone(key: &str, data: &TzRow) -> bool {
  let expiry = 15 * 60;
  redis_set_data::<TzRow>(key, data, expiry)
}

pub fn redis_get_timezone(key: &str) -> Option<TzRow> {
  redis_get_data::<TzRow>(key)
}

pub fn  redis_set_strings(key: &str, data: &Vec<String>) -> bool {
  redis_set_data::<Vec<String>>(key, data, 0)
}

pub fn redis_get_strings(key: &str) -> Option<Vec<String>> {
  redis_get_data::<Vec<String>>(key)
}

pub fn  redis_set_weather(key: &str, data: &WeatherReport) -> bool {
  let expiry = 30 * 60;
  redis_set_data::<WeatherReport>(key, data, expiry)
}

pub fn redis_get_weather(key: &str) -> Option<WeatherReport> {
  redis_get_data::<WeatherReport>(key)
}

pub fn  redis_set_poi(key: &str, data: &Vec<PlaceOfInterest>) -> bool {
  let expiry = 31 * 24 * 60 * 60;
  redis_set_data::<Vec<PlaceOfInterest>>(key, data, expiry)
}

pub fn redis_get_poi(key: &str) -> Option<Vec<PlaceOfInterest>> {
  redis_get_data::<Vec<PlaceOfInterest>>(key)
}

pub fn  redis_set_wiki_summaries(key: &str, data: &Vec<WikipediaSummary>) -> bool {
  let expiry = 3 * 31 * 24 * 60 * 60;
  redis_set_data::<Vec<WikipediaSummary>>(key, data, expiry)
}

pub fn redis_get_wiki_summaries(key: &str) -> Option<Vec<WikipediaSummary>> {
  redis_get_data::<Vec<WikipediaSummary>>(key)
}

pub fn redis_set_astro_data(key: &str, data: &AstroData) -> bool {
  // only 30 minutes
  let expiry = 30 * 60;
  redis_set_data::<AstroData>(key, data, expiry)
}

pub fn redis_get_astro_data(key: &str) -> Option<AstroData> {
  redis_get_data::<AstroData>(key)
}

pub fn redis_set_addresses_checked(pc: &str) -> bool {
  let expiry = 183 * 24 * 60 * 60;
  let key = format!("address_check_{}", pc.replace(" ", "_"));
  redis_set_data::<u8>(&key, &1, expiry)
}

pub fn redis_addresses_have_been_checked(pc: &str) -> bool {
  let key = format!("address_check_{}", pc.replace(" ", "_"));
  let stored = redis_get_data::<u8>(&key);
  stored.is_some()
}

pub fn redis_get_data<T>(key: &str) -> Option<T>
  where T: DeserializeOwned + Serialize {
  if let Some(result) = redis_get_opt_string(key) {
    if let Ok(items) = serde_json::from_str::<T>(&result) {
      return Some(items);
    }
  }
  None
}

pub fn  redis_set_data<'a, T:Serialize + Deserialize<'a>>(key: &str, data: &T, expiry: usize) -> bool {
  if let Ok(mut connection) =  redis_client() {
    if let Ok(value) = serde_json::to_string(&data) {
      let stored = if expiry > 0  {
        connection.set_ex::<String,String,String>(key.to_string(), value, expiry)
      } else {
        connection.set::<String,String,String>(key.to_string(), value)
      };
      if let Ok(_result) = stored {
          return true;
      }
    }
  }
  false
}