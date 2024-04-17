use axum::{
  extract,
  http::StatusCode,
  response::IntoResponse,
  Json
};
use mongodb::Client;
use serde_json::json;
use string_patterns::PatternReplace;

use crate::{
  addresses::get_remote_addresses, astro::get_astro_data,
  common::{build_store_key_from_geo, is_valid_date_string, GeoParams, PostParams},
  fetchers::{fetch_pc_zone, fetch_pc_zones, fetch_pcs, update_pc_addresses},
  geonames::{fetch_poi_cached, fetch_weather_cached, fetch_wiki_entries_cached},
  geotime::{get_geotz_data, get_place_lookup, get_tz_data},
  models::{Geo, GeoTimeInfo, LocationInfo, PcZone, PlaceRow, SimplePlace},
  simple_iso::timestamp_from_string,
  store::{
    redis_addresses_have_been_checked, redis_get_astro_data, redis_get_geo_nearby, redis_get_pc_results, redis_get_pc_zones, redis_get_place_rows, redis_get_timezone, redis_set_astro_data, redis_set_geo_nearby, redis_set_pc_results, redis_set_pc_zones, redis_set_place_rows, redis_set_timezone
  }
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
      let cache_key = format!("tz_info_{}_{}_{}", zn_opt.unwrap_or(""), geo.to_approx_key(3), dt_opt.clone().unwrap_or("a".to_string()));
      let mut time_opt = redis_get_timezone(&cache_key);
      let is_cached = time_opt.is_some();
      if !is_cached {
        time_opt =  get_tz_data(geo_opt, zn_opt, dt_opt.clone().as_deref()).await;
      }
      if let Some(mut time) = time_opt {
        if is_cached {
          let ts_opt = if let Some(dt) = dt_opt.clone() {
            timestamp_from_string(&dt)
          } else {
            None
          };
          time.update_time(ts_opt);
        } else {
          redis_set_timezone(&cache_key, &time);
        }
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
    let mut is_uk = false;
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
      if let Some(cc) = geo_item.cc {
        is_uk = cc.starts_with("GB") || cc.starts_with("UK");
      }
    }
    let mut rows: Vec<PcZone> = vec![];
    if is_uk {
      let limit = 7;
    let km = 15.0;
    let ck = build_store_key_from_geo("pzones", geo, Some(km), Some(limit));
      rows = redis_get_pc_zones(&ck);
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
          let has_been_checked = redis_addresses_have_been_checked(pc);
          if !has_been_checked {
            let addresses_opt = get_remote_addresses(pc).await;
            if let Some(addresses) = addresses_opt {
              update_pc_addresses(&client, pc, &addresses).await;
              first.add_addresses(&addresses);
              redis_set_pc_zones(&ck, &rows);
            }
          }
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

pub async fn show_astro_data(query: extract::Query<GeoParams>) -> impl IntoResponse {
  let geo_opt = query.to_geo_opt();
  let mut ts_opt: Option<i64> = None;
  if let Some(dt) = query.dt.clone() {
    ts_opt = timestamp_from_string(&dt);
  }
  let mut response = json!({ "valid": false });
  if let Some(geo) = geo_opt {
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
      response = json!({ "valid": true, "astro": astro });
    }
  }
  (StatusCode::OK, Json(response))
}

pub async fn show_place_lookup(query: extract::Query<GeoParams>) -> impl IntoResponse {
  let search = if let Some(place_str) = query.place.clone() {
    place_str
  } else if let Some(search_str) = query.search.clone() {
    search_str
  } else {
    "".to_owned()
  };
  let fuzzy_opt = query.fuzzy;
  let cc_opt = query.cc.clone();
  let mut response = json!([]);
  if search.len() > 1 {
    let mut key_parts = vec!["lookup".to_string(), search.to_lowercase().pattern_replace_ci(r#"\s+"#, "_")];
    if let Some(cc) = cc_opt.clone() {
      key_parts.push(cc);
    }
    if let Some(fz) = fuzzy_opt.clone() {
      key_parts.push(fz.to_string());
    }
    let cache_key = key_parts.join("_");
    let rows_opt = redis_get_place_rows(&cache_key);
    let is_cached = rows_opt.is_some();
    let mut has_uncached_results = false;
    let mut rows: Vec<PlaceRow> = Vec::new();
    if !is_cached {
      if let Some(results ) = get_place_lookup(&search, cc_opt, fuzzy_opt).await {
        rows = results;
        has_uncached_results = true;
      }
    } else {
      if let Some(c_rows) = rows_opt {
        rows = c_rows;
      }
    }
    if has_uncached_results {
      redis_set_place_rows(&cache_key, &rows);
    }
    response = json!(rows);
  }
  (StatusCode::OK, Json(response))
}

pub async fn show_timezone(query: extract::Query<GeoParams>) -> impl IntoResponse {
  let mut status = StatusCode::NOT_ACCEPTABLE;
  let mut response = json!({"valid": false });
  if let Some(geo) = query.to_geo_opt() {
    let mut dt_opt: Option<String> = None;
     // Clone query.dt outside the inner if let block
     let dt = query.dt.clone();
     if let Some(ds) = dt {
         if is_valid_date_string(&ds) {
             dt_opt = Some(ds); // Assign ds directly, not as a reference
         }
    }
    let zn_opt = query.zn.clone();
    let zn_key = zn_opt.clone().unwrap_or("".to_owned());
    let geo_opt = Some(geo);
    let cache_key = format!("tz_info_{}_{}_{}", zn_key, geo.to_approx_key(3), dt_opt.clone().unwrap_or("a".to_string()));
    let mut time_opt = redis_get_timezone(&cache_key);
    let is_cached = time_opt.is_some();
    if !is_cached {
      time_opt =  get_tz_data(geo_opt, zn_opt.as_deref(), dt_opt.clone().as_deref()).await;
    }
    if let Some(mut time) = time_opt {
      if is_cached {
        let ts_opt = if let Some(dt) = dt_opt.clone() {
          timestamp_from_string(&dt)
        } else {
          None
        };
        time.update_time(ts_opt);
      } else {
        redis_set_timezone(&cache_key, &time);
      }
      status = StatusCode::OK;
      response = json!(time);
    }
  }
  (status, Json(response))
}