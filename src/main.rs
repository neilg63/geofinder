

extern crate chrono;
extern crate redis;

mod db;
mod extractors;
mod bson_extractors;
mod models;
mod fetchers;
mod common;
mod store;
mod geotime;
mod addresses;
mod geonames;

mod handlers;

//use std::io;
use std::net::SocketAddr;
use std::time::Duration;

use axum::{
  http::{self, HeaderMap},
    http::{header, HeaderValue},
    routing::{get, post},
    Router, middleware::{Next, self},
};
use dotenv::dotenv;
use mongodb::{
    options::ClientOptions,
    Client,
};
use tower_http::{
    limit::RequestBodyLimitLayer,
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
    timeout::TimeoutLayer,
    cors::CorsLayer
};
use crate::common::{welcome, handler_404};
use crate::handlers::{get_nearest_pcs,get_gtz,fetch_and_update_addresses, get_weather_report,get_places_of_interest,get_nearby_wiki_summaries,get_geo_data};
use crate::db::*;
// use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // initialize tracing
    dotenv().ok();

    let database_config = DatabaseConfig::new();
    let mut client_options = ClientOptions::parse(database_config.uri).await.unwrap();
    client_options.connect_timeout = database_config.connection_timeout;
    client_options.max_pool_size = database_config.max_pool_size;
    client_options.min_pool_size = database_config.min_pool_size;
    // the server will select the algorithm it supports from the list provided by the driver
    client_options.compressors = database_config.compressors;
    let client = Client::with_options(client_options).unwrap();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(welcome))
        .route("/postcodes", get(get_nearest_pcs))
        .route("/gtz", get(get_gtz))
        .route("/addresses", post(fetch_and_update_addresses))
        .route("/weather", get(get_weather_report))
        .route("/places-of-interest", get(get_places_of_interest))
        .route("/wiki-summaries", get(get_nearby_wiki_summaries))
        .route("/geo-codes", post(get_geo_data))
        .layer(CorsLayer::permissive())
        // timeout requests after 10 secs, returning 408 status code
        .layer(TimeoutLayer::new(Duration::from_secs(10)))
        // don't allow request bodies larger than 1024 bytes, returning 413 status code
        .layer(RequestBodyLimitLayer::new(1024))
        .layer(TraceLayer::new_for_http())
        .layer(SetResponseHeaderLayer::if_not_present(
            header::SERVER,
            HeaderValue::from_static("rust-axum"),
        ));
    let app = app.fallback(handler_404).with_state(client);
    let env_port = if let Ok(port_ref) = dotenv::var("PORT") { port_ref } else { "3000".to_owned() };
    let port = if let Ok(p) = u16::from_str_radix(&env_port, 10) { p } else { 3000 };
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::debug!("listening on {}", addr);
    println!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}