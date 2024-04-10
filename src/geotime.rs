use serde_json::{Map, Value};
use mongodb::Client;
use crate::{common::{get_gtz_url, is_valid_zone_name}, fetchers::get_nearest_pc_info, models::{Geo, GeoNearby, GeoTimeInfo, TzRow}};



pub async fn get_geotz_data(client: &Client, geo: Geo, date_opt: Option<&str>) -> Option<GeoTimeInfo> {
  let req_client = reqwest::Client::new();
  let loc = geo.to_string();
  let mut query_params = vec![
    ("loc", loc.as_str()),
  ];
  if date_opt.is_some() {
    query_params.push(("dt", date_opt.unwrap_or("")));
  }
  let uri = format!("{}/geotz", get_gtz_url());

  let result = req_client.get(&uri)
    .query(&query_params).send()
    .await
    .expect("failed to get response")
    .text()
    .await;
  if let Ok(result_string) = result {
    let data: Map<String, Value> = serde_json::from_str(&result_string).unwrap();
    if let Some(place_data) = data.get("place") {
      if let Some(pd) = place_data.as_object() {
        let mut place = GeoNearby::new(pd);
        if let Some(pc_info) = get_nearest_pc_info(client, geo).await {
          place.add_pc(&pc_info);
        }
        if let Some(time_data) = data.get("time") {
          if let Some(td)  = time_data.as_object() {
            let time = TzRow::new(td);
            return Some(GeoTimeInfo::new(place, time));
          }
        }
      }
    }
  }
  None   
}

pub async fn get_tz_data(geo_opt: Option<Geo>, zn_opt: Option<&str>, date_opt: Option<&str>) -> Option<TzRow> {
  let client = reqwest::Client::new();
  let opt_str = if let Some(zn) = zn_opt {
    zn.to_string()
  } else if let Some(geo) = geo_opt {
    geo.to_string()
  } else {
    "".to_string()
  };
  let opt_key = if zn_opt.is_some() { "zn" } else { "loc" };
  let mut query_params = vec![
    (opt_key, opt_str.as_str()),
  ];
  let valid = geo_opt.is_some() || is_valid_zone_name(&opt_str);
  if date_opt.is_some() {
    query_params.push(("dt", date_opt.unwrap_or("")));
  }
  if valid {
    let uri = format!("{}/timezone", get_gtz_url());
    let result = client.get(&uri)
      .query(&query_params).send()
      .await
      .expect("failed to get response")
      .text()
      .await;
    if let Ok(result_string) = result {
      let data: Map<String, Value> = serde_json::from_str(&result_string).unwrap();
      if data.contains_key("abbreviation") {
        let mut tz_data = TzRow::new(&data);
        if let Some(geo) = geo_opt {
          tz_data.calc_solar_offset(geo.lng);
        }
        return Some(tz_data);
      }
    }
  }
  None   
}