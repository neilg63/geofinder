use bson::{doc, Document, Bson};
use chrono::{DateTime, Utc, SecondsFormat};
use serde_json::Value;

pub fn extract_string(doc: &Document, key: &str) -> String {
  doc.get_str(key).unwrap_or("").to_string()
}

/* pub fn extract_str<'a>(doc: &'a Document, key: &str) -> &'a str {
  doc.get_str(key).unwrap_or("")
}

pub fn extract_datetime(doc: &Document, key: &str) -> String {
  if let Ok(dt_val) = doc.get_datetime(key) {
    dt_val.to_string()
  } else {
    "".to_string()
  }
}

pub fn extract_isodt(doc: &Document, key: &str) -> Option<DateTime<Utc>> {
  if let Ok(dt_val) = doc.get_datetime(key) {
    Some(dt_val.to_chrono())
  } else {
    None
  }
}
pub fn extract_isodt_as_string(doc: &Document, key: &str) -> Option<String> {
  if let Some(dt) = extract_isodt(doc, key) {
    Some(dt.to_rfc3339_opts(SecondsFormat::Millis, true))
  } else {
    None
  }
}

pub fn extract_bool(doc: &Document, key: &str, def_val: bool) -> bool {
  doc.get_bool(key).unwrap_or(def_val)
} */

pub fn extract_i32(doc: &Document, key: &str) -> i32 {
  if let Ok(vl) = doc.get_i32(key) {
    vl
  } else {
    if let Ok(vl) = doc.get_f64(key) {
      if vl > i32::MAX as f64 {
        i32::MAX
      } else if vl < i32::MIN as f64 {
        i32::MIN
      } else {
        vl as i32
      }
    } else {
      0i32
    }
  }
}


pub fn extract_i64(doc: &Document, key: &str) -> i64 {
  if let Ok(vl) = doc.get_i64(key) {
    vl
  } else {
    if let Ok(vl) = doc.get_i32(key) {
      vl as i64
    } else if let Ok(vl) = doc.get_f64(key) {
      vl as i64
    } else {
      0i64
    }
  }
}

pub fn extract_i8(doc: &Document, key: &str) -> i8 {
  extract_i32(doc, key) as i8
}

pub fn extract_u8(doc: &Document, key: &str) -> u8 {
  let val_i32 = extract_i32(doc, key);
  if val_i32 >= 0 && val_i32 <= u8::MAX as i32{
    val_i32 as u8
  } else {
    0u8
  }
}

pub fn extract_u16(doc: &Document, key: &str) -> u16 {
  let val_i32 = extract_i32(doc, key);
  if val_i32 >= 0 && val_i32 <= u16::MAX as i32{
    val_i32 as u16
  } else {
    0u16
  }
}

pub fn extract_u32(doc: &Document, key: &str) -> u32 {
  let val_i32 = extract_i32(doc, key);
  if val_i32 >= 0 && val_i32 <= u32::MAX as i32 {
    val_i32 as u32
  } else {
    0u32
  }
}


pub fn extract_f64(doc: &Document, key: &str) -> f64 {
  let f_val = doc.get_f64(key).unwrap_or(0f64);
  if f_val != 0f64 {
    f_val
  } else {
    extract_i32(doc, key) as f64
  }
}

pub fn extract_f32(doc: &Document, key: &str) -> f32 {
  extract_f64(doc, key) as f32
}

pub fn extract_f64_or(doc: &Document, key: &str, def_val: f64) -> f64 {
  doc.get_f64(key).unwrap_or(def_val)
}


pub fn extract_vec(doc: &Document, key: &str) -> Vec<Bson> {
  doc.get_array(key).unwrap_or(&vec![]).to_owned()
}

pub fn extract_sub_doc(doc: &Document, key: &str) -> Option<Document> {
  if let Ok(d) = doc.get_document(key) {
    Some(d.to_owned())
  } else {
    None
  }
}

pub fn extract_custom(doc: &Document, key: &str) -> Option<Value> {
  if let Some(bson) = doc.get(key) {
    Some(bson.clone().into_relaxed_extjson())
  } else {
    None
  }
}

pub fn extract_as_vec(doc: &Document, key: &str) -> Vec<Document> {
  let rows = doc.get_array(key).unwrap_or(&vec![]).to_owned();
  let mut items: Vec<Document> = Vec::new();
  if rows.len() > 0 {
    for row in rows {
      if let Some(doc) = row.as_document() {
        items.push(doc.to_owned());
      }
    }
  }
  items
}


pub fn extract_strings(doc: &Document, key: &str) -> Vec<String> {
  extract_vec(doc, key).into_iter().map(|bs| bs.as_str().unwrap_or("").to_string() ).collect()
}

pub fn extract_ints(doc: &Document, key: &str) -> Vec<i64> {
  extract_vec(doc, key).into_iter().map(|bs| bs.as_i64().unwrap_or(0i64)).collect()
}

pub fn extract_i32s(doc: &Document, key: &str) -> Vec<i32> {
  extract_ints(doc, key).into_iter().map(|v| v as i32).collect()
}

pub fn extract_i8s(doc: &Document, key: &str) -> Vec<i8> {
  extract_ints(doc, key).into_iter().map(|v| v as i8).collect()
}

fn extract_int_as_f64(bs: Bson) -> f64 {
  let i_val_opt = bs.as_i32();
  if let Some(i_val) = i_val_opt {
    i_val as f64
  } else {
    0f64
  }
}

pub fn extract_floats(doc: &Document, key: &str) -> Vec<f64> {
  extract_vec(doc, key).into_iter().map(|bs| {
    let fl_opt = bs.as_f64();
    if let Some(fl) = fl_opt {
      if fl != 0f64 {
        fl
      } else {
        extract_int_as_f64(bs)
      }
    } else {
      extract_int_as_f64(bs)
    }
  }).collect()
}

pub fn extract_id(doc: &Document, key: &str) -> String {
  let id = doc.get_object_id(key);
  if let Ok(id_val) = id {
    id_val.to_string()
  } else {
    "".to_string()
  }
}

pub fn extract_doc(doc: &Document, key: &str) -> Document {
  let item = doc.get_document(key);
  if let Ok(d) = item {
    d.to_owned()
  } else {
    doc!{}
  }
}
