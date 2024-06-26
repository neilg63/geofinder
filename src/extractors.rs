use serde_json::*;

pub fn extract_f64_from_value_map(row: &Map<String, Value>, key: &str) -> f64 {
  match row.get(key) {
      Some(num_str_val) => match num_str_val {
          Value::String(num_str) =>  num_str.parse::<f64>().unwrap_or(0f64),
          Value::Number(num_ref) =>  num_ref.as_f64().unwrap_or(0f64),
          _ => 0f64,
      },
      _ => 0f64,
  }
}

pub fn extract_optional_i64_from_value_map(row: &Map<String, Value>, key: &str) -> Option<i64> {
  match row.get(key) {
      Some(num_str_val) => match num_str_val {
          Value::String(num_str) =>  if let Ok(pv) = num_str.parse::<i64>() {
            Some(pv)
          } else {
            None
          },
          Value::Number(num_ref) =>  num_ref.as_i64(),
          _ => None,
      },
      _ => None,
  }
}

pub fn extract_optional_string_from_value_map(row: &Map<String, Value>, key: &str) -> Option<String> {
  match row.get(key) {
      Some(num_str_val) => match num_str_val {
          Value::String(num_str) =>  Some(num_str.to_owned()),
          Value::Number(num_ref) =>  Some(num_ref.to_string()),
          _ => None,
      },
      _ => None,
  }
}

pub fn extract_string_from_value_map(row: &Map<String, Value>, key: &str) -> String {
  if let Some(str_val) = extract_optional_string_from_value_map(row, key) {
    str_val
  } else {
    "".to_string()
  }
}

pub fn extract_display_strings_from_value_map(row: &Map<String, Value>, key: &str) -> Vec<String> {
  if let Some(mp) = row.get(key) {
    if let Some(items) = mp.as_array() {
      return items.into_iter().filter_map(|item| {
        if let Some(inner_item) = item.as_object() {
          extract_optional_string_from_value_map(inner_item, "Display")
        } else {
          None
        }
      }).collect::<Vec<String>>();
    }
  }
  vec![]
}

pub fn extract_u32_from_value_map(row: &Map<String, Value>, key: &str) -> u32 {
  match row.get(key) {
      Some(num_val) => match num_val {
          Value::String(num_str) =>  num_str.parse::<u32>().unwrap_or(0u32),
          Value::Number(num_ref) =>  num_ref.as_i64().unwrap_or(0i64) as u32,
          _ => 0u32,
      },
      _ => 0u32,
  }
}

pub fn extract_bool_from_value_map(row: &Map<String, Value>, key: &str, def_val: bool) -> bool {
  match row.get(key) {
      Some(bool_val) => match bool_val {          
          Value::Number(num_ref) =>  num_ref.as_i64().unwrap_or(0i64) > 0,
          Value::Bool(bool_ref) =>  bool_ref.to_owned(),
          _ => def_val,
      },
      _ => def_val,
  }
}

pub fn extract_from_key_f64_values(data: &Map<String, Value>, key: &str) -> Vec<f64> {
  let mut positions: Vec<f64> = vec![];
  if data.contains_key("values") {
    if let Some(values) = data.get("values") {
      if let Some(items) = values.as_array() {
        for item in items.into_iter() {
          if let Some(obj) = item.as_object() {
            let v = extract_f64_from_value_map(obj, key);
            positions.push(v);
          }
        }
      }
    }
  }
  positions
}

pub fn extract_inner_i64(data: &Map<String, Value>, key_1: &str, key_2: &str) -> i64 {
  let mut val_i64 = 0i64;
  if let Some(inner) = data.get(key_1) {
    if let Some(item) = inner.as_object() {
      val_i64 = extract_optional_i64_from_value_map(item, key_2).unwrap_or(0);
    }
  }
  val_i64
}

pub fn extract_inner_f64(data: &Map<String, Value>, key_1: &str, key_2: &str) -> f64 {
  let mut val_f64 = 0f64;
  if let Some(inner) = data.get(key_1) {
    if let Some(item) = inner.as_object() {
      val_f64 = extract_f64_from_value_map(item, key_2);
    }
  }
  val_f64
}