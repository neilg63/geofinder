use mongodb::{
    bson::{doc, Document}, options::{AggregateOptions, FindOptions}, Client, Collection
};
use futures::stream::StreamExt;
// use simple_string_patterns::ToSegments;

use crate::{common::{build_store_key_from_geo, get_db_name}, models::{Geo, PcInfo, PcRow, PcZone}, store::{redis_get_pc_results, redis_set_pc_results}};

pub async fn find_records(client: &Client, coll_name: &str, limit: u64, skip: u64, filter_options: Option<Document>, fields: Option<Vec<&str>>) -> Vec<Document> {
  let db_name = get_db_name();
  let collection: Collection<Document> =
      client.database(&db_name).collection::<Document>(coll_name);
  let max = if limit > 0 { limit as i64 } else { 10000000i64 };
  let mut projection: Option<Document> = None;
  if let Some(field_list) = fields {
      let mut doc = doc! {};
      for field in field_list {
          doc.insert(field, 1);
      }
      projection = Some(doc);
  }
  let find_options = FindOptions::builder().projection(projection).skip(skip).limit(max).build();
  let cursor_r = collection
      .find(
          filter_options,
          find_options,
      )
      .await;
   if let Ok(cursor) = cursor_r {
    let results: Vec<mongodb::error::Result<Document>> = cursor.collect().await;
    let mut rows: Vec<Document> = Vec::new();
    if results.len() > 0 {
        for item in results {
            if let Ok(row) = item {
                rows.push(row);
            }
        }
    }
    rows
   } else {
      vec![]
   }
}

pub async fn fetch_record(client: &Client, coll_name: &str,
    filter_options: Option<Document>
) -> Option<Document> {
    let records = find_records(client, coll_name, 1, 0, filter_options, None).await;
    let mut result: Option<Document> = None;
    if records.len() > 0 {
        for row in records {
            result = Some(row);
        }
    }
    result
}

/* pub async fn fetch_records(client: &Client, coll_name: &str, filter_options: Option<Document>, fields: Option<Vec<&str>>) -> Vec<Document> {
    find_records(client, coll_name, 0, 0, filter_options, fields).await
} */

pub async fn update_record(client: &Client, coll_name: &str, filter_options: &Document, values: &Document) -> bool {
  let update = doc ! { "$set": values.to_owned() };
  let db_name = get_db_name();
  let collection: Collection<Document> = client.database(&db_name).collection::<Document>(coll_name);
  let cursor_r = collection
      .update_one(
          filter_options.to_owned(),
          update,
          None
      )
      .await;
  cursor_r.is_ok()
}

pub async fn fetch_aggregated_with_options(client: &Client, coll_name: &str, pipeline: Vec<Document>, options: Option<AggregateOptions>) -> Vec<Document> {
  let db_name = get_db_name();
  let coll: Collection<Document> = client
        .database(&db_name)
        .collection::<Document>(coll_name);
    let cursor = coll
        .aggregate(pipeline, options)
        .await
        .expect("could not load data.");
    let results: Vec<mongodb::error::Result<Document>> = cursor.collect().await;
    let mut rows: Vec<Document> = Vec::new();
    if results.len() > 0 {
        for item in results {
            if let Ok(row) = item {
                rows.push(row);
            }
        }
    }
    rows
}

pub async fn fetch_aggregated(client: &Client, coll_name: &str, pipeline: Vec<Document>) -> Vec<Document> {
  fetch_aggregated_with_options(client, coll_name, pipeline, None).await
}

pub fn build_geo_search(geo: Geo, km: f64) -> Document {
  let max_distance_metres = km * 1000f64;
  doc! {
      "$geoNear": {
          "near": {
              "type": "Point",
              "coordinates": [geo.lng, geo.lat]
          },
          "minDistance": 0,
          "maxDistance": max_distance_metres,
          "spherical": true,
          "distanceField": "distance"
      }
    }
}

pub async fn fetch_pcs(client: &Client, geo: Geo, km: f64, limit: u32) -> Vec<PcRow> {
  let geo_search = build_geo_search(geo, km);
  let mut pipeline = vec![geo_search];
  let projection = doc! {
    "_id": 0,
    "lat": 1,
    "lng": 1,
    "pc": 1,
    "c": 1,
    "cv": 1,
    "d": 1,
    "lc": 1,
    "w": 1,
    "distance": 1
  };
  pipeline.push(doc! { "$project": projection } );
  let limit_u32 = if limit < 2 { 2 } else if limit > 1000  { 1000  } else { limit };
  pipeline.push(doc! { "$limit":  limit_u32 } );
  let rows = fetch_aggregated(client, "zones", pipeline).await;
  if rows.len() > 0 {
    rows.into_iter().map(|row| PcRow::new(&row)).collect::<Vec<PcRow>>()
  } else {
    vec![]
  }
}

pub async fn fetch_pc_zones(client: &Client, geo: Geo, km: f64, limit: u32) -> Vec<PcZone> {
  let geo_search = build_geo_search(geo, km);
  let mut pipeline = vec![geo_search];
  let projection = doc! {
    "_id": 0,
    "lat": 1,
    "lng": 1,
    "alt": 1,
    "pc": 1,
    "addresses": 1,
    "c": 1,
    "cv": 1,
    "d": 1,
    "lc": 1,
    "w": 1,
    "wc": 1,
    "e": 1,
    "n": 1,
    "gr": 1,
    "distance": 1,
    "modifiedAt": 1,
  };
  pipeline.push(doc! { "$project": projection } );
  let limit_u32 = if limit < 2 { 2 } else if limit > 1000  { 1000  } else { limit };
  pipeline.push(doc! { "$limit":  limit_u32 } );
  let rows = fetch_aggregated(client, "zones", pipeline).await;
  if rows.len() > 0 {
    rows.into_iter().map(|row| PcZone::new(&row)).collect::<Vec<PcZone>>()
  } else {
    vec![]
  }
}

pub async fn get_nearest_pc_info(client: &Client, geo: Geo) -> Option<PcInfo> {
  let ck = build_store_key_from_geo("pc", geo, Some(15.0), Some(1), 6);
  let mut rows = redis_get_pc_results(&ck);
  let mut info: Option<PcInfo> = None;
  if rows.len() < 1 {
    rows = fetch_pcs(&client, geo, 15.0, 1).await;
    if rows.len() > 0 {
      redis_set_pc_results(&ck, &rows);
    }
  }
  if rows.len() > 0 {
    if let  Some(row) = rows.get(0)  {
      info = Some(row.as_info());
    }
  }
  info
}

pub async fn update_pc_addresses(client: &Client, pc: &str, addresses: &[String]) -> bool {
  let query = doc ! { "pc": pc };
  let data = doc ! { "addresses": addresses };
  update_record(client, "zones",&query, &data).await
}


pub async fn fetch_pc_zone(client: &Client, pc: &str) -> Option<PcZone> {
  let filter = Some(doc ! { "pc": pc });
  let result = fetch_record(client, "zones",filter).await;
  result.map(|item| PcZone::new(&item))
}

/* fn extract_string_from_vec(vals: &Vec<String>, index: usize) -> String {
  vals.get(index).unwrap_or(&"".to_string()).to_owned()
} */

/* 
pub async fn get_update_lines(client: &Client) -> usize {
  let text_lines = read_lines("/Users/neil/apps/laravel/geotime-zone-extra-sources/dg_ng_export.tsv");
  let mut counter: usize = 0;
  let mut index = 0;
  let start = 0;
  for line in text_lines {
    if index >= start {
      let cells = line.to_parts("\t");
      if cells.len() > 4 {
        if let Some(pc) = cells.get(0) {
          if pc.len() > 3 {
            let updated = update_incomplete_pc_zone(client, &cells).await;
            if updated {
              counter += 1;
            }
          }
        }
      }
      if counter > 10000 {
        break;
      }
    }
    index += 1;
  }
  counter
}

pub async fn update_incomplete_pc_zone(client: &Client, vals: &Vec<String>) -> bool {
  if let Some(first) = vals.get(0) {
    let pc = first.to_owned().clone();
    let query = doc ! { "pc": pc, "w": { "$exists": false } };
    let n = extract_string_from_vec(vals, 1);
    let e = extract_string_from_vec(vals, 2);
    let w = extract_string_from_vec(vals, 3);
    let cy = extract_string_from_vec(vals, 4);
    let d = extract_string_from_vec(vals, 5);
    let gr = extract_string_from_vec(vals, 6);
    let data = doc ! { "n": n, "e": e, "w": w, "cy": cy, "d": d, "gr": gr };
    return update_record(client, "zones",&query, &data).await;
  }
  false
} */