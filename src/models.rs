use serde::{Deserialize, Serialize};
use serde_json::*;
use bson::{doc, Document};
use crate::extractors::*;
use crate::bson_extractors::*;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeoNearby {
    pub lng: f64,
    pub lat: f64,
    pub name: String,
    pub toponym: String,
    pub fcode: String,
    pub distance: f64,
    pub pop: u32,
    #[serde(rename="adminName")]
    pub admin_name: String,
    pub region: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cc: Option<String>,
    #[serde(rename="countryName")]
    pub country_name: String,
    #[serde(rename="zoneName",skip_serializing_if = "Option::is_none")]
    pub zone_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pc: Option<PcInfo>
}

impl GeoNearby {
  pub fn new(row: &Map<String, Value>) -> GeoNearby {
    let lng = extract_f64_from_value_map(&row, "lng");
    let lat = extract_f64_from_value_map(&row, "lat");
    let name = extract_string_from_value_map(&row, "name");
    let region = extract_string_from_value_map(&row, "region");
    let admin_name = extract_string_from_value_map(&row, "adminName");
    let country_name = extract_string_from_value_map(&row, "countryName");
    let cc = extract_optional_string_from_value_map(&row, "cc");
    let toponym = extract_string_from_value_map(&row, "toponym");
    let fcode = extract_string_from_value_map(&row, "fcode");
    let pop = extract_u32_from_value_map(&row, "population");
    let distance = extract_f64_from_value_map(&row, "distance");
    let zone_name = extract_optional_string_from_value_map(&row, "zoneName");
    GeoNearby { 
      lng,
      lat,
      name,
      toponym,
      fcode,
      distance,
      pop,
      admin_name,
      region,
      cc,
      country_name,
      zone_name,
      pc: None
    }
  }

  pub fn add_pc(&mut self, info: &PcInfo) {
    self.pc = Some(info.to_owned());
  }

}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PcRow {
  pub lat: f64,
  pub lng: f64,
  pub c: String,
  pub cy: String,
  pub d: String,
  pub pc : String,
  pub lc: String,
  pub w: String,
  pub distance: f64
}

impl PcRow {
  pub fn new(dc: &Document) -> PcRow {
    let distance = extract_f64(dc, "distance");
    let lat = extract_f64(dc, "lat");
    let lng = extract_f64(dc, "lng");
    let c = extract_string(dc, "c");
    let cy = extract_string(dc, "cy");
    let pc = extract_string(dc, "pc");
    let d = extract_string(dc, "d");
    let lc = extract_string(dc, "lc");
    let w = extract_string(dc, "w");
    PcRow {
      lat,
      lng,
      c,
      cy,
      d,
      pc,
      lc,
      w,
      distance
    }
  }

  pub fn as_info(&self) -> PcInfo {
    PcInfo::new(&self.pc, self.distance)
  }

}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PcInfo {
  pub v: String,
  pub m: f64,
}

impl PcInfo {
  pub fn new(code: &str, metres: f64) -> Self {
    PcInfo {
      v: code.to_string(),
      m: metres
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct Geo {
  pub lat: f64,
  pub lng: f64,
  pub alt: f64
}

impl Geo {
  pub fn new(lat: f64, lng: f64, alt: f64) -> Geo {
    Geo {
      lat,
      lng,
      alt
    }
  }

  pub fn simple(lat: f64, lng: f64) -> Geo {
    Geo {
      lat,
      lng,
      alt: 10f64
    }
  }

}

impl ToString for Geo {
  fn to_string(&self) -> String {
    let alt_str = if self.alt < 0.0 || self.alt > 10.0 {
      format!(",{:0}", self.alt)
    } else {
      "".to_owned()
    };
    format!("{},{}{}", self.lat, self.lng,alt_str)
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TzRow {
  abbreviation: String,
  #[serde(rename="countryCode")]
  country_code: String,
  dst:bool,
  #[serde(rename="gmtOffset")]
  gmt_offset: i64,
  #[serde(rename="localDt")]
  local_dt: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  period: Option<TzPeriod>,
  #[serde(rename="refUnix")]
  ref_unix: i64,
  #[serde(rename="solarUtcOffset")]
  solar_utc_offset: i64,
  utc: String,
  #[serde(rename="weekDay")]
  week_day: u8,
  zone_name: String,
}

impl TzRow {
  pub fn new(row: &Map<String, Value>) -> TzRow {
    let abbreviation = extract_string_from_value_map(&row, "abbreviation");
    let country_code = extract_string_from_value_map(&row, "countryCode");
    let local_dt = extract_string_from_value_map(&row, "localDt");
    let dst = extract_bool_from_value_map(&row, "dst", false);
    let solar_utc_offset = extract_optional_i64_from_value_map(&row, "solarUtcOffset").unwrap_or(0);
    let ref_unix = extract_optional_i64_from_value_map(&row, "refUnix").unwrap_or(0);
    let mut week_day = 0;
    if let Some(wd) = row.get("weekDay") {
      if let Some(wd) = wd.as_object() {
        week_day = extract_u32_from_value_map(wd, "iso") as u8;
      }
    }
    let utc = extract_string_from_value_map(&row, "utc");
    let gmt_offset = extract_optional_i64_from_value_map(&row, "gmtOffset").unwrap_or(0);
    let mut period: Option<TzPeriod> = None;
    if let Some(p_item) = row.get("period") {
      if let Some(p_map) = p_item.as_object() {
        let p = TzPeriod::new(p_map);
        if p.start.is_some() || p.end.is_some() {
          period = Some(p);
        }
      }
    };
    let zone_name = extract_string_from_value_map(&row, "zoneName");
    TzRow {
      abbreviation,
      country_code,
      gmt_offset,
      local_dt,
      dst,
      utc,
      period,
      ref_unix,
      solar_utc_offset,
      week_day,
      zone_name
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TzPeriod {
  pub start: Option<i64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub end: Option<i64>,
  #[serde(rename="nextGmtOffset",skip_serializing_if = "Option::is_none")]
  pub next_gmt_offset: Option<i64>,
}

impl TzPeriod {
  pub fn new(row: &Map<String, Value>) -> TzPeriod {
    let start = extract_optional_i64_from_value_map(&row, "start");
    let end = extract_optional_i64_from_value_map(&row, "end");
    let next_gmt_offset = extract_optional_i64_from_value_map(&row, "next_gmt_offset");
    TzPeriod {
      start,
      end,
      next_gmt_offset,
    }
  }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeoTimeInfo {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub place: Option<GeoNearby>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub time: Option<TzRow>,
  pub cached: bool,
  pub valid: bool,
}

impl GeoTimeInfo {
  pub fn new(place: GeoNearby, time: TzRow) -> Self {
    GeoTimeInfo {
      place: Some(place),
      time: Some(time),
      cached: false,
      valid: true
    }
  }

  pub fn new_geoplace(place: GeoNearby) -> Self {
    GeoTimeInfo {
      place: Some(place),
      time: None,
      cached: false,
      valid: true
    }
  }

  pub fn set_cached(&mut self) {
    self.cached = true;
  }

  pub fn set_time(&mut self, time: TzRow) {
    self.time = Some(time);
  }

}