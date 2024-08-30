#[macro_use]
extern crate diesel;

pub mod database;
pub mod jobs;
pub mod logger;
pub mod models;
pub mod schema;

use axum::{routing::post, Router};
use jobs::{create_job_handler, list_jobs_handler};
use logger::init_logger;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    init_logger().unwrap();

    let app = Router::new().route("/jobs", post(create_job_handler).get(list_jobs_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 7878));

    log::info!("🚀 Server running on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
