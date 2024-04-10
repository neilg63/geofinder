use std::collections::HashMap;
use std::path::Path;
use axum::http::{HeaderMap, HeaderValue};
use reqwest::header::{CONTENT_TYPE, USER_AGENT};
use serde_json::{Map, Value};
use crate::{common::{get_addresses_url,is_valid_uk_postcode, read_lines}, extractors::extract_display_strings_from_value_map};
use string_patterns::PatternFilter;
use rand::prelude::*;
use crate::store::{redis_get_strings, redis_set_strings};

const DEFAULT_SPIDER_USER_AGENT_STRING: &'static str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/117.0.0.0 Safari/537.36";

fn match_user_agent_file_name() -> Option<String> {
  if let Ok(file_ref) = dotenv::var("USER_AGENT_STRINGS_FILE") {
    if Path::new(&file_ref).exists() {
      return Some(file_ref);
    }
  }
  None
}

fn get_user_agent_lines() -> Vec<String> {
  let ck = "user_agents";
  let lines_opt = redis_get_strings(ck);
  if let Some(lines) = lines_opt {
    lines
  } else {
    if let Some(user_agent_list_file) = match_user_agent_file_name() {
      let text_lines = read_lines(&user_agent_list_file);
      if text_lines.len() > 5 {
        redis_set_strings(ck, &text_lines);
      }
      text_lines
    } else {
      vec![]
    }
  }
}

fn get_random_ua_string() -> String {
  let mut ua_str= DEFAULT_SPIDER_USER_AGENT_STRING.to_string();
  let lines = get_user_agent_lines();
  let num_lines = lines.len();
  if num_lines > 1 {
    let random: usize = rand::thread_rng().gen();
    let rand_index = random % num_lines;
    if let Some(ln) = lines.get(rand_index) {
      ua_str = ln.to_owned();
    }
  }
  ua_str
}

fn build_headers() -> HeaderMap {
  let mut hm = HeaderMap::new();
  let ua = get_random_ua_string();
  hm.insert(USER_AGENT, ua.parse().unwrap());
  hm.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
  hm
}

pub async fn get_remote_addresses(pc: &str) -> Option<Vec<String>> {
  let client = reqwest::Client::new();
  let pc_code = pc.trim().to_uppercase();
  let valid = is_valid_uk_postcode(&pc_code);
  
  if valid {
    let mut map = HashMap::new();    
    map.insert("Query", pc_code.clone());
    map.insert("CountryIsoCode", "GBR".to_string());
    let uri = get_addresses_url();
    let hm = build_headers();
    let result = client.post(&uri)
    .headers(hm)
      .json(&map)
      .send()
      .await;
    if let Ok(resp) = result {
      if let Ok(data) = resp.json::<Map<String, Value>>().await {
        if data.contains_key("Data") {
          let addresses = extract_display_strings_from_value_map(&data, "Data");
          let filtered_address = addresses.pattern_filter_ci(&pc_code);
          return Some(filtered_address);
        }
      }
    }
  }
  None   
}

