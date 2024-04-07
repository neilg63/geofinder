use std::{ops::Add, str::FromStr};
use bson::Bson;
use chrono::Duration;
use mongodb::{
    bson::{doc, Document, oid::ObjectId},
    Client,
    Collection,
    options::{FindOptions, AggregateOptions},
};
use futures::stream::StreamExt;
use serde_json::{json, Map, Value};
use string_patterns::*;

use crate::{common::{build_store_key_from_geo, get_db_name}, models::{Geo, GeoNearby, PcInfo, PcRow, TzRow}, store::{redis_get_pc_results, redis_set_pc_results}};

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

pub async fn fetch_records(client: &Client, coll_name: &str, filter_options: Option<Document>, fields: Option<Vec<&str>>) -> Vec<Document> {
    find_records(client, coll_name, 0, 0, filter_options, fields).await
}

pub async fn fetch_aggregated_with_options(client: &Client, coll_name: &str, pipeline: Vec<Document>, options: Option<AggregateOptions>) -> Vec<Document> {
  let db_name = get_db_name();
  let coll: Collection<Document> = client
        .database(&db_name)
        .collection::<Document>(coll_name);


    let cursor = coll
        .aggregate(pipeline, options)
        .await
        .expect("could not load users data.");
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

pub async fn get_nearest_pc_info(client: &Client, geo: Geo) -> Option<PcInfo> {
  let ck = build_store_key_from_geo("pc", geo, Some(15.0), Some(1));
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

