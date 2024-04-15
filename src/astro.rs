use serde_json::{Map, Value};
use crate::{common::get_astro_url, models::{AstroData, Geo}};

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