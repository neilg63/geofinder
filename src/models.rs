use core::num;
use std::collections::HashSet;
use std::thread::current;
use chrono::DateTime;
use chrono::Datelike;
use chrono::SecondsFormat;
use chrono::Utc;
use julian_day_converter::*;
use bson::datetime;
use serde::{Deserialize, Serialize};
use serde_json::*;
use bson::{doc, Document};
use crate::common::natural_tz_offset_from_utc;
use crate::extractors::*;
use crate::bson_extractors::*;
use crate::simple_iso::*;


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

  pub fn to_simple(&self) -> SimplePlace {
    SimplePlace::new(self.lat, self.lng, &self.name)
  }

  pub fn to_places(&self) -> Vec<SimplePlace> {
    vec![self.to_simple()]
  }

  pub fn to_states(&self) -> Vec<SimplePlace> {
    vec![
      SimplePlace::new(self.lat, self.lng, &self.admin_name),
      SimplePlace::new(self.lat, self.lng, &self.region),
      SimplePlace::new(self.lat, self.lng, &self.country_name)
    ]
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



  pub fn to_approx_key(&self, places: u8) -> String {
    let multiple = 10f64.powf(places as f64);
    let lat_str = ((self.lat * multiple).round() / multiple).to_string();
    let lng_str = ((self.lng * multiple).round() / multiple).to_string();
    format!("{}_{}", lat_str, lng_str)
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

  pub fn calc_solar_offset(&mut self, lng: f64) {
    self.solar_utc_offset = natural_tz_offset_from_utc(lng);
  }

  pub fn get_next_period_ts(&self) -> i64 {
    if let Some(period) = self.period {
      period.start.unwrap_or(0)
    } else {
      0
    }
  }

  pub fn get_next_period_offset(&self) -> i64 {
    if let Some(period) = self.period {
      period.next_gmt_offset.unwrap_or(self.gmt_offset)
    } else {
      0
    }
  }

  pub fn update_time(&mut self, ts_opt: Option<i64>) {
    let ref_dt = if let Some(ts_val) = ts_opt {
      DateTime::from_timestamp(ts_val,0).unwrap_or(Utc::now())
    } else {
      Utc::now()
    };
    let ts = ref_dt.timestamp();
    self.ref_unix = ts_opt.unwrap_or(ref_dt.timestamp());
    self.utc = ref_dt.to_simple_iso();
    if ts >= self.get_next_period_ts() {
      self.gmt_offset = self.get_next_period_offset();
    }
    let offset_ts = ts + self.gmt_offset;
    if let Some(lt) = DateTime::from_timestamp(offset_ts,0) {
      self.local_dt = lt.to_simple_iso();
      self.week_day = lt.weekday().number_from_monday() as u8;
    }
  }

}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PcZone {
  pub pc: String,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  addresses: Vec<String>,
  lat: f64,
  lng: f64,
  alt: f64,
  n: f64,
  e: f64,
  c: String,
  cy: String,
  d: String,
  wc: String,
  cs: String,
  lc: String,
  w: String,
  gr: String,
  #[serde(rename="modifiedAt")]
  modified_at: String,
  dist: f64,
  #[serde(skip_serializing_if = "Option::is_none")]
  pn: Option<String>,
}

impl PcZone {
  pub fn new(dc: &Document) -> PcZone {
    let dist = extract_f64(dc, "distance");
    let lat = extract_f64(dc, "lat");
    let lng = extract_f64(dc, "lng");
    let n = extract_f64(dc, "n");
    let e = extract_f64(dc, "e");
    let alt = extract_f64(dc, "alt");
    let wc = extract_string(dc, "wc");
    let c = extract_string(dc, "c");
    let cy = extract_string(dc, "cy");
    let cs = extract_string(dc, "cs");
    let gr = extract_string(dc, "gr");
    let pc = extract_string(dc, "pc");
    let d = extract_string(dc, "d");
    let lc = extract_string(dc, "lc");
    let w = extract_string(dc, "w");
    let modified_at =  extract_datetime(dc, "modifiedAt");
    let addresses = extract_strings(dc, "addresses");
    PcZone {
      pc,
      addresses,
      lat,
      lng,
      alt,
      n,
      e,
      c,
      cy,
      d,
      wc,
      cs,
      lc,
      w,
      gr,
      dist,
      modified_at,
      pn: None
    }
  }

  pub fn has_addresses(&self) -> bool {
    self.addresses.len() > 0
  }

  pub fn add_addresses(&mut self, addresses: &[String]) {
    self.addresses = addresses.to_vec();
  }

  pub fn add_pn(&mut self, place_name: &str) {
    self.pn = Some(place_name.to_string());
  }

}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SimplePlace {
  lng: f64,
  lat: f64,
  name: String,
}

impl SimplePlace {
  pub fn new(lat: f64, lng: f64, name: &str) -> Self {
    SimplePlace {
      lat,
      lng,
      name: name.to_string(),
    }
  }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaceOfInterest {
  lng: f64,
  lat: f64,
  distance: f64,
  name: String,
  #[serde(rename="typeClass")]
  type_class: String,
  #[serde(rename="typeName")]
  type_name: String,
}

impl PlaceOfInterest {
  pub fn new(row: Map<String, Value>) -> Self {
    let lng = extract_f64_from_value_map(&row, "lng");
    let lat = extract_f64_from_value_map(&row, "lat");
    let distance = extract_f64_from_value_map(&row, "distance");
    let mut name = extract_string_from_value_map(&row, "name").trim().to_string();
    let type_class = extract_string_from_value_map(&row, "typeClass");
    let type_name = extract_string_from_value_map(&row, "typeName");
    if name.len() < 1 {
      name = type_name.clone();
    }
    PlaceOfInterest { 
        lng,
        lat,
        distance,
        name,
        type_class,
        type_name,
    }
  }

  pub fn get_name(&self) -> String {
    self.name.clone()
  }
}

pub fn build_pois(data: Map<String, Value>) -> Vec<PlaceOfInterest> {
  let mut rows:Vec<PlaceOfInterest> = vec![];
  let mut names: HashSet<String> = HashSet::new();
  if data.contains_key("poi") {
    rows = match &data["poi"] {
      Value::Array(items) => {
        let mut new_rows: Vec<PlaceOfInterest> = vec![];
        for row in items {
          match row {
            Value::Object(row_map) => {
              let new_row = PlaceOfInterest::new(row_map.clone());
              let name = new_row.get_name();
              if names.contains(&name) == false {
                names.insert(name);
                new_rows.push(new_row);
              }
            },
            _ => ()
          }
        }
        new_rows
      },
      _ => Vec::new(),
    };
  }
  rows
}


pub fn build_wiki_summaries(data: Map<String, Value>) -> Vec<WikipediaSummary> {
  let mut rows:Vec<WikipediaSummary> = vec![];
  if data.contains_key("geonames") {
    rows = match &data["geonames"] {
      Value::Array(items) => {
        let mut new_rows: Vec<WikipediaSummary> = vec![];
        for row in items {
          match row {
            Value::Object(row_map) => {
              let new_row = WikipediaSummary::new(row_map.clone());
              new_rows.push(new_row);
            },
            _ => ()
          }
        }
        new_rows
      },
      _ => Vec::new(),
    };
  }
  rows
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WeatherReport {
  lat: f64,
  lng: f64,
  datetime: String,
  temperature: f64,
  humidity: f64,
  #[serde(rename="windSpeed")]
  wind_speed: f64,
  #[serde(rename="dewPoint")]
  dew_point: f64,
  #[serde(rename="stationName")]
  station_name: String,
  clouds: String
}

impl WeatherReport {
  pub fn new(row: Map<String, Value>) -> Self {
    let lng = extract_f64_from_value_map(&row, "lng");
    let lat = extract_f64_from_value_map(&row, "lat");
    let datetime = extract_string_from_value_map(&row, "datetime");
    let temperature = extract_f64_from_value_map(&row, "temperature");
    let humidity = extract_f64_from_value_map(&row, "humidity");
    let wind_speed = extract_f64_from_value_map(&row, "windSpeed");
    let dew_point = extract_f64_from_value_map(&row, "dewPoint");
    let station_name = extract_string_from_value_map(&row, "stationName");
    let clouds = extract_string_from_value_map(&row, "clouds");
    WeatherReport { 
        lat,
        lng,
        datetime,
        temperature,
        humidity,
        wind_speed,
        dew_point,
        station_name,
        clouds
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WikipediaSummary {
  pub lat: f64,
  pub lng: f64,
  pub summary: String,
  pub title: String,
  pub elevation: f64,
  pub distance: f64,
  pub rank: i64,
  pub lang: String,
  #[serde(rename="wikipediaUrl")]
  pub wikipedia_url: String
}

impl WikipediaSummary {
  pub fn new(row: Map<String, Value>) -> Self {
    let lng = extract_f64_from_value_map(&row, "lng");
    let lat = extract_f64_from_value_map(&row, "lat");
    let summary = extract_string_from_value_map(&row, "summary");
    let title = extract_string_from_value_map(&row, "title");
    let lang = extract_string_from_value_map(&row, "lang");
    let elevation = extract_f64_from_value_map(&row, "elevation");
    let distance = extract_f64_from_value_map(&row, "distance");
    let rank = extract_optional_i64_from_value_map(&row, "rank").unwrap_or(-1);
    let wikipedia_url = extract_string_from_value_map(&row, "wikipediaUrl");
    WikipediaSummary { 
        lat,
        lng,
        summary,
        title,
        elevation,
        distance,
        rank,
        lang,
        wikipedia_url
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocationInfo {
  pub matched: bool,
  pub valid: bool,
  #[serde(rename="hasWeather")]
  pub has_weather: bool,
  #[serde(rename="hasPoi")]
  pub has_poi: bool,
  #[serde(rename="hasWikiEntries")]
  pub has_wiki_entries: bool,
  #[serde(rename="hasNearestAddress")]
  pub has_nearest_address: bool,
  #[serde(rename="hasPCs")]
  pub has_pcs: bool,
  pub num: u32,
  pub zone: Option<PcZone>,
  pub places: Vec<SimplePlace>,
  pub states: Vec<SimplePlace>,
  pub surrounding: Vec<PcZone>,
  pub cached: bool,
  pub weather: Option<WeatherReport>,
  pub poi: Vec<PlaceOfInterest>,
  pub wikipedia: Vec<WikipediaSummary>
}

impl LocationInfo {
  pub fn new(zones: Vec<PcZone>, places: Vec<SimplePlace>, states: Vec<SimplePlace>, weather: Option<WeatherReport>, poi: Vec<PlaceOfInterest>, wikipedia: Vec<WikipediaSummary>) -> Self {
    let valid = places.len() > 0;
    let matched = places.len() > 0;
    let has_poi = poi.len() > 0;
    let num = zones.len() as u32;
    let zone = zones.get(0).map(|z| z.to_owned());
    let has_pcs = zones.len() > 0;
    let surrounding = if num > 0 {
      (&zones[1..].to_vec()).to_owned()
    } else {
      vec![]
    };
    let has_nearest_address = if let Some(zn) = zone.clone() {
      zn.has_addresses()
    } else {
      false
    };
    let has_weather = weather.is_some();
    let has_wiki_entries = wikipedia.len() > 0;
    LocationInfo {
      valid,
      matched,
      has_weather,
      has_wiki_entries,
      has_nearest_address,
      has_pcs,
      has_poi,
      num,
      zone,
      surrounding,
      places,
      states,
      weather,
      poi,
      wikipedia,
      cached: false
    }

  }

  pub fn set_cached(&mut self) {
    self.cached = true;
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AscendantData {
  pub lng: f64,
  pub positions: Vec<f64>,
}

impl AscendantData {
  pub fn new(data: &Map<String, Value>) -> Self {
    let mut positions: Vec<f64> = extract_from_key_f64_values(data, "as");
    let index = extract_u32_from_value_map(data, "currentIndex") as usize;
    let lng = positions.get(index).map(|v| v.to_owned()).unwrap_or(0.0);
    AscendantData  {
      lng,
      positions,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoonPhase {
  pub num: u8,
  pub ts: i64
}
impl MoonPhase {
  pub fn new(data: &Map<String, Value>) -> Self {
    let num_val = extract_u32_from_value_map(data, "num");
    let num = if num_val < 5 {
      num_val
    } else {
      0
    } as u8;
    let jd = extract_f64_from_value_map(data, "jd");
    let ts = julian_day_converter::julian_day_to_unixtime(jd);
    MoonPhase {
      num,
      ts
    }
  }

  pub fn valid(&self) -> bool {
    self.num > 0
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoonData {
  pub lng: f64,
  pub positions: Vec<f64>,
  pub phase: u8,
  pub sun_angle: f64,
  pub waxing: bool,
  pub phases: Vec<MoonPhase>
}

impl MoonData {
  pub fn new(data: &Map<String, Value>) -> Self {
    let mut positions: Vec<f64> = extract_from_key_f64_values(data, "mo");
    let index = extract_u32_from_value_map(data, "currentIndex") as usize;
    let lng = positions.get(index).map(|v| v.to_owned()).unwrap_or(0.0);
    let mut phase = 0;
    let mut sun_angle = 0.0;
    let mut waxing = false;
    let mut phases: Vec<MoonPhase> = vec![];
    if data.contains_key("moon") {
      if let Some(inner) = data.get("moon") {
        if let Some(moon) = inner.as_object() {
          phase = extract_u32_from_value_map(moon, "phase") as u8;
          waxing = extract_bool_from_value_map(moon, "waxing", false);
          sun_angle = extract_f64_from_value_map(moon, "sunAngle");
          let phases_key = if moon.contains_key("phases") {
            "phases"
          } else if moon.contains_key("nextPhases") {
            "nextPhases"
          } else {
            "-"
          };
          if phases_key.len() > 1 {
            if let Some(values) = moon.get(phases_key) {
              if let Some(items) = values.as_array() {
                for item in items.into_iter() {
                  if let Some(obj) = item.as_object() {
                    let mp = MoonPhase::new(obj);
                    if mp.valid() {
                      phases.push(mp);
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
    MoonData  {
      lng,
      positions,
      phase,
      sun_angle,
      waxing,
      phases
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SunData {
  pub lng: f64,
  pub positions: Vec<f64>,
  pub rise: Option<i64>,
  pub set: Option<i64>,
  pub mc: Option<i64>,
  pub ic: Option<i64>,
  pub min: f64,
  pub max: f64,
}

impl SunData {
  pub fn new(data: &Map<String, Value>) -> Self {
    let positions: Vec<f64> = extract_from_key_f64_values(data, "su");
    let index = extract_u32_from_value_map(data, "currentIndex") as usize;
    let lng = positions.get(index).map(|v| v.to_owned()).unwrap_or(0.0);
    let mut rise: Option<i64> = None;
    let mut set: Option<i64> = None;
    let mut mc: Option<i64> = None;
    let mut ic: Option<i64> = None;
    let mut min: f64 = 0.0;
    let mut max: f64 = 0.0;
    if data.contains_key("sunRiseSets") {
      if let Some(inner) = data.get("sunRiseSets") {
        if let Some(rows) = inner.as_array() {
          for row in rows.into_iter() {
            if let Some(obj) = row.as_object() {
              let v = extract_f64_from_value_map(obj, "value");
              let key = extract_string_from_value_map(obj, "key");
              match key.as_str() {
                "rise" => {
                  rise = Some(julian_day_to_unixtime(v));
                },
                "set" => {
                  set = Some(julian_day_to_unixtime(v));
                },
                "mc" => {
                  mc = Some(julian_day_to_unixtime(v));
                },
                "ic" => {
                  ic = Some(julian_day_to_unixtime(v));
                },
                "min" => {
                  min = v;
                },
                "max" => {
                  max = v;
                },
                _ => {}
              }
            }
          }
        }
      }
    }
    SunData  {
      lng,
      positions,
      rise,
      set,
      mc,
      ic,
      min,
      max
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AstroData {
  pub start: i64,
  pub time: i64,
  pub end: i64,
  #[serde(rename="intervalSecs")]
  pub interval_secs: u32,
  pub sun: Option<SunData>,
  pub ascendant: Option<AscendantData>,
  pub moon: Option<MoonData>,
  #[serde(rename="ageSecs",skip_serializing_if = "Option::is_none")]
  pub age_secs: Option<i64>
}

impl AstroData {
  pub fn new(data: &Map<String, Value>) -> Self {
    let ascendant: Option<AscendantData> = Some(AscendantData::new(data));
    let moon: Option<MoonData> = Some(MoonData::new(data));
    let sun: Option<SunData> = Some(SunData::new(data));
    let time: i64 = extract_inner_i64(data, "date", "unix");
    let start: i64 = extract_inner_i64(data, "start", "unix");
    let end: i64 = extract_inner_i64(data, "end", "unix");
    let interval_days = extract_inner_f64(data, "interval", "days");
    let i_secs_f64 = interval_days * 86400.0;
    let interval_secs = if i_secs_f64 <= 4_294_967_295.0 && i_secs_f64 >= 0.0 {
      (i_secs_f64).round() as u32
    } else {
      0
    };
    AstroData {
      start,
      time,
      end,
      interval_secs,
      sun,
      moon,
      ascendant,
      age_secs: None
    }
  }

  pub fn set_age(&mut self) {
    let curr_ts = Utc::now().timestamp();
    self.age_secs = Some(curr_ts - self.time);
  }

} 