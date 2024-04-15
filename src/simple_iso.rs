use chrono::{DateTime, SecondsFormat, TimeZone, Utc, Local};

pub trait SimpleISO8601 {
  fn to_simple_iso(&self) -> String;
}

impl<Tz: TimeZone> SimpleISO8601 for DateTime<Tz> {
  fn to_simple_iso(&self) -> String {
    self.to_rfc3339_opts(SecondsFormat::Secs, true).replace("Z", "")
  }
}

pub fn timestamp_from_string(date_str: &str) -> Option<i64> {
  if let Ok(ndt) = julian_day_converter::iso_fuzzy_string_to_datetime(date_str) {
    Some(DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc).timestamp())
  } else {
    None
  }
}