#[macro_use]
extern crate diesel;

mod logger;
mod models;
mod schema;

use axum::http::StatusCode;
use axum::{routing::post, Json, Router};
use logger::init_logger;
use models::{Job, NewJob};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use uuid::Uuid;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenvy::dotenv;
use std::env;

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let connection = SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    return connection;
}

#[tokio::main]
async fn main() {
    init_logger().unwrap();

    let app = Router::new().route("/jobs", post(create_job_handler).get(list_jobs_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 7878));

    log::info!("🚀 Server running on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

#[derive(Serialize)]
struct JobDto {
    id: String,
    last_error: Option<String>,
    max_retries: i32,
    name: String,
    payload: String,
    retry_count: i32,
    status: String,
}

fn transform_job(job: Job) -> JobDto {
    JobDto {
        id: job.id,
        last_error: job.last_error,
        max_retries: job.max_retries,
        name: job.name,
        payload: job.payload,
        retry_count: job.retry_count,
        status: job.status,
    }
}

#[derive(Deserialize)]
struct NewJobData {
    max_retries: Option<i32>,
    name: String,
    payload: String,
}

#[derive(Serialize)]
struct CreateJobResponse {
    job: JobDto,
}

async fn create_job_handler(
    Json(new_job_data): Json<NewJobData>,
) -> Result<Json<CreateJobResponse>, (StatusCode, String)> {
    let name = new_job_data.name.trim();
    let payload = new_job_data.payload.trim();
    let max_retries = new_job_data.max_retries.unwrap_or(3);

    if name.is_empty() {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            "name cannot be empty.".into(),
        ));
    }

    if max_retries < 0 {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            "max_retries must be greater than or equal to 0.".into(),
        ));
    }

    if max_retries > 64 {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            "max_retries must be less than or equal to 64.".into(),
        ));
    }

    let connection = &mut establish_connection();

    let id = Uuid::new_v4().to_string();

    let new_job = NewJob {
        id: &id,
        max_retries,
        name,
        retry_count: 0,
        payload,
        status: String::from("pending"),
    };

    let result = diesel::insert_into(schema::jobs::table)
        .values(&new_job)
        .execute(connection);

    if result.is_err() {
        log::error!("Failed to create job. {:?}", result);

        return Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create job.".into(),
        ));
    }

    use self::schema::jobs::dsl::jobs;

    let job = jobs.find(id).select(Job::as_select()).first(connection);

    match job {
        Err(_) => {
            return Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to fetch job.".into(),
            ));
        }
        Ok(job) => {
            log::info!("Created {} job.", job.name);

            let response = CreateJobResponse {
                job: transform_job(job),
            };

            return Ok(Json(response));
        }
    }
}

#[derive(Serialize)]
struct ListJobsResponse {
    jobs: Vec<JobDto>,
}

async fn list_jobs_handler() -> Result<Json<ListJobsResponse>, (StatusCode, String)> {
    let connection = &mut establish_connection();

    let results = self::schema::jobs::dsl::jobs
        .select(Job::as_select())
        .load(connection);

    match results {
        Ok(jobs) => {
            let jobs = jobs.into_iter().map(transform_job).collect();
            let response = ListJobsResponse { jobs };

            Ok(Json(response))
        }
        Err(e) => {
            log::error!("Failed to fetch jobs. {}", e);

            return Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to fetch jobs.".into(),
            ));
        }
    }
}
