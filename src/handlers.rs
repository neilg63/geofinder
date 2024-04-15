use axum::{
  extract,
  http::StatusCode,
  response::IntoResponse,
  Json
};
use mongodb::Client;
use serde_json::json;

use crate::{
  addresses::get_remote_addresses, 
  common::{build_store_key_from_geo, is_valid_date_string, GeoParams, PostParams},
  fetchers::{fetch_pc_zone, fetch_pc_zones, fetch_pcs, update_pc_addresses},
  geonames::{fetch_poi, fetch_poi_cached, fetch_weather, fetch_weather_cached, fetch_wiki_entries, fetch_wiki_entries_cached},
  geotime::{get_geotz_data, get_tz_data}, models::{Geo, GeoTimeInfo, LocationInfo, SimplePlace},
  store::{redis_get_geo_nearby, redis_get_pc_results, redis_get_pc_zones, redis_get_poi, redis_get_weather, redis_get_wiki_summaries, redis_set_geo_nearby, redis_set_pc_results, redis_set_pc_zones, redis_set_poi, redis_set_weather, redis_set_wiki_summaries}
};


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
        redis_set_pc_results(&ck, &rows);
      }
    } else {
      cached = true;
    }
    let response = json!({ "valid": true, "cached": cached, "rows": rows });
    (StatusCode::OK, Json(response))
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
         if is_valid_date_string(&ds) {
             dt_opt = Some(ds); // Assign ds directly, not as a reference
         }
    }
    let ck = build_store_key_from_geo("place", geo, None, None);
    let mut data: Option<GeoTimeInfo> = None;
    let geo_data = redis_get_geo_nearby(&ck);
    if let Some(gdata) = geo_data {
      let has_zn = gdata.zone_name.is_some();
      let zn_opt = if has_zn { gdata.zone_name.as_deref() } else { None };
      let geo_opt = Some(geo);
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


pub async fn fetch_and_update_addresses(extract::State(client): extract::State<Client>, query: extract::Json<PostParams>) -> impl IntoResponse {
  if let Some(pc) = query.pc.clone() {
    let pc_zone_opt = fetch_pc_zone(&client, &pc).await;
    if let Some(mut pc_zone) = pc_zone_opt {
      if !pc_zone.has_addresses() {
        let addresses_opt = get_remote_addresses(&pc).await;
        if let Some(addresses) = addresses_opt {
          update_pc_addresses(&client, &pc, &addresses).await;
          pc_zone.add_addresses(&addresses);
        }
      }
      let response = json!(pc_zone);
      return (StatusCode::OK, Json(response));
    } 
  }
  let response = json!({ "valid": false });
  (StatusCode::NOT_ACCEPTABLE, Json(response))
}

pub async fn get_weather_report(query: extract::Query<GeoParams>) -> impl IntoResponse {
  let mut response = json!({ "valid": false });
  let mut status = StatusCode::NOT_ACCEPTABLE;
  if let Some(geo) = query.to_geo_opt() {
    let (weather_opt, cached) = fetch_weather_cached(geo).await;
    status = if weather_opt.is_some() {
      StatusCode::OK
    } else {
      StatusCode::NOT_FOUND
    };
    if let Some(weather)=  weather_opt {
      response = json!({ "valid": true, "cached": cached, "weather": weather });
    } else {
      response = json!({ "valid": true, "cached": false });
    }
  }
  (status, Json(response))
}

pub async fn get_places_of_interest(query: extract::Query<GeoParams>) -> impl IntoResponse {
  let mut response = json!({ "valid": false });
  let mut status = StatusCode::NOT_ACCEPTABLE;
  if let Some(geo) = query.to_geo_opt() {
    let (poi_opt, cached) = fetch_poi_cached(geo).await;
    status = if poi_opt.is_some() { 
      StatusCode::OK
    } else {
      StatusCode::NOT_FOUND
    };
    if let Some(poi) = poi_opt {
      response = json!({ "valid": true, "cached": cached, "items": poi });
    }
    
  }
  (status, Json(response))
}

pub async fn get_nearby_wiki_summaries(query: extract::Query<GeoParams>) -> impl IntoResponse {
  let mut response = json!({ "valid": false });
  let mut status = StatusCode::NOT_ACCEPTABLE;
  if let Some(geo) = query.to_geo_opt() {
    let (items_opt, cached) = fetch_wiki_entries_cached(geo).await;
    status = if items_opt.is_some() {
      StatusCode::OK
    } else {
      StatusCode::NOT_FOUND
    };
    if let Some(items) = items_opt {
      response = json!({ "valid": true, "cached": cached, "items": items });
    }
  }
  (status, Json(response))
}

pub async fn get_geo_data(extract::State(client): extract::State<Client>, query: extract::Json<PostParams>) -> impl IntoResponse {
  if let Some(lat) = query.lat {
    let geo = Geo::new(lat, query.lng.unwrap_or(0.0), 10.0);
    let ck = build_store_key_from_geo("place", geo, None, None);
    let mut pn = "".to_string();
    let mut geo_data = redis_get_geo_nearby(&ck);
    let mut places: Vec<SimplePlace> = vec![];
    let mut states: Vec<SimplePlace> = vec![];
    if geo_data.is_none() {
      if let Some(gtz_data)= get_geotz_data(&client, geo, None).await {
        if let Some(place) = gtz_data.place.clone() {
          redis_set_geo_nearby(&ck, &place);
          geo_data = Some(place);
        }
      }
    }
    if let Some(geo_item) = geo_data {
      places = geo_item.to_places();
      states = geo_item.to_states();
      pn = geo_item.name.clone();
    }
    let limit = 20;
    let km = 5.0;
    let ck = build_store_key_from_geo("pzones", geo, Some(km), Some(limit));
    let mut rows = redis_get_pc_zones(&ck);
    if rows.len() < 1 {
      rows = fetch_pc_zones(&client, geo, km, limit).await;
      if rows.len() > 0 {
        redis_set_pc_zones(&ck, &rows);
      }
    }
    if let Some(first) = rows.get_mut(0) {
      first.add_pn(&pn);
      if !first.has_addresses() {
        let pc = first.pc.as_str();
        let addresses_opt = get_remote_addresses(pc).await;
        
        if let Some(addresses) = addresses_opt {
          update_pc_addresses(&client, pc, &addresses).await;
          first.add_addresses(&addresses);
          redis_set_pc_zones(&ck, &rows);
        }
      }
    }
    
    let (weather, _weather_cached) = fetch_weather_cached(geo).await;
    let (poi_opt, _poi_cached) = fetch_poi_cached(geo).await;
    let poi = poi_opt.unwrap_or(vec![]);
    let (wiki_items_opt, _wiki_cached) = fetch_wiki_entries_cached(geo).await;
    let wikipedia = wiki_items_opt.unwrap_or(vec![]);
    let result = LocationInfo::new(rows, places, states, weather, poi, wikipedia);
    let response = json!(result);
    return (StatusCode::OK, Json(response));
  }
  let response = json!({ "valid": false });
  (StatusCode::NOT_ACCEPTABLE, Json(response))
}

/* pub async fn read_pc_zone_updates(extract::State(client): extract::State<Client>, query: extract::Query<GeoParams>) -> impl IntoResponse {
  let num_updated = get_update_lines(&client).await;
  let response = json!({ "valid": false, "num_updated": num_updated });
  (StatusCode::OK, Json(response))
} */