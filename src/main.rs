#[macro_use]
extern crate diesel;

pub mod database;
pub mod jobs;
pub mod logger;
pub mod models;
pub mod schema;

use axum::{routing::post, Router};
use dotenvy::dotenv;
use jobs::{
    claim_job_handler, create_failed_run_handler, create_job_handler,
    create_successful_run_handler, list_jobs_handler,
};
use logger::init_logger;
use std::env;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    dotenv().ok();

    init_logger().unwrap();

    let app = Router::new()
        .route("/jobs", post(create_job_handler).get(list_jobs_handler))
        .route("/claim", post(claim_job_handler))
        .route("/successful-run", post(create_successful_run_handler))
        .route("/failed-run", post(create_failed_run_handler));

    let port = env::var("PORT").unwrap_or("7878".to_string());
    let port = port.parse::<u16>().unwrap_or(7878);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    log::info!("🚀 Server running on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
