use serde_json::{Map, Value};
use crate::{common::get_astro_url, models::{AstroData, Geo}, simple_iso::timestamp_from_string, store::{redis_get_astro_data, redis_set_astro_data}};

async fn fetch_core_astro(geo: Geo, ts_opt: Option<i64>) -> Option<Map<String, Value>> {
  let req_client = reqwest::Client::new();
  let loc = geo.to_string();
  let mut query_params = vec![
    ("loc", loc.as_str()),
    ("full", "1"),
    ("bodies", "su,mo"),
  ];
  let mut jd_string: Box<String> = Box::new("".to_owned());
  if let Some(ts) = ts_opt {
    let jd = julian_day_converter::unixtime_to_julian_day(ts);
    jd_string = Box::new(jd.to_string());
    query_params.push(("jd", &jd_string));
  }
  let uri = format!("{}/{}", get_astro_url(), "ascendant");
  let result = req_client.get(&uri)
    .query(&query_params).send()
    .await
    .expect("failed to get response")
    .text()
    .await;
  if let Ok(result_string) = result {
    if let Ok(json) = serde_json::from_str(&result_string) {
      return Some(json);
    }
  }
  None
}

pub async fn get_astro_data(geo: Geo, ts_opt: Option<i64>) -> Option<AstroData> {
  let data_opt = fetch_core_astro(geo, ts_opt).await;
  if let Some(data) = data_opt {
    if data.contains_key("date") && data.contains_key("values") {
      let astro = AstroData::new(&data);
      return  Some(astro);
    }
  }
  None
}

pub async fn get_astro_data_cached(geo: Geo, dt_opt: Option<String>) -> Option<AstroData> {
  let mut ts_opt: Option<i64> = None;
  if let Some(dt) = dt_opt.clone() {
    ts_opt = timestamp_from_string(&dt);
  }
  let ts_key = if ts_opt.is_some() {
    (ts_opt.unwrap_or(0) / 1800).to_string()
  } else {
    "c".to_owned()
  };
  let key = format!("astro_data_{}_{}", geo.to_approx_key(2), ts_key);
  let mut astro_opt = redis_get_astro_data(&key);
  let is_cached = astro_opt.is_some();
  if !is_cached {
    astro_opt = get_astro_data(geo, ts_opt).await; 
  }
  if let Some(mut astro) = astro_opt {
    if is_cached {
      astro.set_age();
    } else {
      redis_set_astro_data(&key, &astro);
    }
    return Some(astro);
  }
  None
}