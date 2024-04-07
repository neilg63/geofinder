use axum::{
  extract,
  http::StatusCode,
  response::IntoResponse,
  Json
};
use mongodb::Client;
use serde_json::json;
use string_patterns::PatternMatch;

use crate::{common::{build_store_key_from_geo, GeoParams}, fetchers::fetch_pcs, geotime::{get_geotz_data, get_tz_data}, models::GeoTimeInfo, store::{redis_get_geo_nearby, redis_get_pc_results, redis_set_geo_nearby, redis_set_pc_results}};


pub async fn get_nearest_pcs(extract::State(client): extract::State<Client>, query: extract::Query<GeoParams>) -> impl IntoResponse {
  if let Some(geo) = query.to_geo_opt() {
    let km = query.km.unwrap_or(10.0);
    let limit = query.limit.unwrap_or(10);
    let ck = build_store_key_from_geo("pc", geo, Some(km), Some(limit));
    let mut rows = redis_get_pc_results(&ck);
    let mut cached = false;
    if rows.len() < 1 {
      rows = fetch_pcs(&client, geo, km, limit).await;
      if rows.len() > 0 {
        let st = redis_set_pc_results(&ck, &rows);
      }
    } else {
      cached = true;
    }
    let response = json!({ "valid": true, "cahed": cached, "rows": rows });
    (StatusCode::NOT_ACCEPTABLE, Json(response))
  } else {
    let response = json!({ "valid": false });
    (StatusCode::NOT_ACCEPTABLE, Json(response))
  }
}



pub async fn get_gtz(extract::State(client): extract::State<Client>, query: extract::Query<GeoParams>) -> impl IntoResponse {
  if let Some(geo) = query.to_geo_opt() {
    let mut dt_opt: Option<String> = None;
     // Clone query.dt outside the inner if let block
     let dt = query.dt.clone();
     if let Some(ds) = dt {
         if ds.pattern_match_cs(r#"^\d\d\d\d-[01]\d-[0-3]\d"#) {
             dt_opt = Some(ds); // Assign ds directly, not as a reference
         }
    }
    let ck = build_store_key_from_geo("place", geo, None, None);
    let mut data: Option<GeoTimeInfo> = None;
    let geo_data = redis_get_geo_nearby(&ck);
    if let Some(gdata) = geo_data {
      let has_zn = gdata.zone_name.is_some();
      let zn_opt = if has_zn { gdata.zone_name.as_deref() } else { None };
      let geo_opt = if has_zn { None } else { Some(geo) };
      let time_opt =  get_tz_data(geo_opt, zn_opt, dt_opt.clone().as_deref()).await;
      if let Some(time) = time_opt {
        let mut info = GeoTimeInfo::new(gdata, time);
        info.set_cached();
        data = Some(info);
      } else {
        let mut info = GeoTimeInfo::new_geoplace(gdata);
        info.set_cached();
        data = Some(info);
      }
    } else {
      if let Some(gtz_data)= get_geotz_data(&client, geo, dt_opt.as_deref()).await {
        if let Some(place) = gtz_data.place.clone() {
          redis_set_geo_nearby(&ck, &place);
        }
        data = Some(gtz_data);
      }
    }
    
    let response = json!(data);
    (StatusCode::NOT_ACCEPTABLE, Json(response))
  } else {
    let response = json!({ "valid": false });
    (StatusCode::NOT_ACCEPTABLE, Json(response))
  }
}
