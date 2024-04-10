
use redis::{streams::StreamPendingCountReply, Commands, Connection, RedisResult};

use crate::{models::{GeoNearby, PcRow, TzRow}, store};

pub(crate) fn redis_client() -> RedisResult<Connection> {
  let client = redis::Client::open("redis://127.0.0.1/")?;
  client.get_connection()
}

pub(crate) fn redis_get_opt_string(key: &str) -> Option<String> {
    if let Ok(mut connection) =  redis_client() {
        let result: String = connection.get(key.to_owned()).unwrap_or("".to_owned());
        Some(result)
    } else {
        None
    }
}

pub(crate) fn redis_set_pc_results(key: &str, data: &Vec<PcRow>) -> bool {
  if let Ok(mut connection) =  redis_client() {
      let stored_data: Vec<PcRow> = data.to_owned();
      if let Ok(value) = serde_json::to_string(&stored_data) {
        let store_result = connection.set::<String,String,String>(key.to_string(), value);
          if let Ok(_result) = store_result {
              true
          } else {
              false
          }
      } else {
          false
      }
      
  } else {
      false
  }
}

pub fn redis_get_pc_results(key: &str) -> Vec<PcRow> {
  if let Some(result) = redis_get_opt_string(key) {
      if result.len() > 0 {
          let mds: Vec<PcRow> = serde_json::from_str(&result).unwrap_or(vec![]);
          mds
      } else {
          vec![]
      }
  } else {
      vec![]
  }
}


pub fn  redis_set_geo_nearby(key: &str, data: &GeoNearby) -> bool {
  let mut valid = false;
  if let Ok(mut connection) =  redis_client() {
    if let Ok(value) = serde_json::to_string(&data) {
      if let Ok(_result) = connection.set::<String,String,String>(key.to_string(), value) {
          valid = true;
      }
    }
  }
  valid
}

pub fn redis_get_geo_nearby(key: &str) -> Option<GeoNearby> {
  if let Some(result) = redis_get_opt_string(key) {
    if let Ok(item) = serde_json::from_str::<GeoNearby>(&result) {
        Some(item)
    } else {
        None
    }
  } else {
    None
  }
}

pub fn  redis_set_get_timezone(key: &str, data: &TzRow) -> bool {
  let mut valid = false;
  if let Ok(mut connection) =  redis_client() {
    let stored_data: TzRow = data.clone();
    if let Ok(value) = serde_json::to_string(&stored_data) {
      if let Ok(_result) = connection.set::<String,String,String>(key.to_string(), value) {
          valid = true;
      }
    }
  }
  valid
}

pub fn redis_get_timezone(key: &str) -> Option<TzRow> {
  if let Some(result) = redis_get_opt_string(key) {
    if let Ok(item) = serde_json::from_str::<TzRow>(&result) {
        Some(item)
    } else {
        None
    }
  } else {
    None
  }
}

pub fn  redis_set_strings(key: &str, data: &Vec<String>) -> bool {
  let mut valid = false;
  if let Ok(mut connection) =  redis_client() {
    if let Ok(value) = serde_json::to_string(data) {
      let rs = connection.set::<String,String,String>(key.to_string(), value);
      if let Ok(_result) = rs {
          valid = true;
      }
    }
  }
  valid
}

pub fn redis_get_strings(key: &str) -> Option<Vec<String>> {
  if let Some(result) = redis_get_opt_string(key) {
    if let Ok(item) = serde_json::from_str::<Vec<String>>(&result) {
        Some(item)
    } else {
        None
    }
  } else {
    None
  }
}

