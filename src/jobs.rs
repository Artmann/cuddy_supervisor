use axum::http::StatusCode;
use axum::Json;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::database::establish_connection;
use super::models::{Job, NewJob};
use super::schema;

#[derive(Serialize)]
struct JobDto {
    id: String,
    last_error: Option<String>,
    max_retries: i32,
    name: String,
    payload: String,
    retry_count: i32,
    status: String,
    worker_id: Option<String>,
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
        worker_id: job.worker_id,
    }
}

#[derive(Deserialize)]
pub struct NewJobInput {
    max_retries: Option<i32>,
    name: String,
    payload: String,
}

#[derive(Serialize)]
pub struct CreateJobResponse {
    job: JobDto,
}

pub async fn create_job_handler(
    Json(new_job_input): Json<NewJobInput>,
) -> Result<Json<CreateJobResponse>, (StatusCode, String)> {
    let name = new_job_input.name.trim();
    let payload = new_job_input.payload.trim();
    let max_retries = new_job_input.max_retries.unwrap_or(3);

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
pub struct ListJobsResponse {
    jobs: Vec<JobDto>,
}

pub async fn list_jobs_handler() -> Result<Json<ListJobsResponse>, (StatusCode, String)> {
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

#[derive(Deserialize)]
pub struct ClaimJobInput {
    worker_id: String,
}

#[derive(Serialize)]
pub struct ClaimJobResponse {
    job: Option<JobDto>,
}

pub async fn claim_job_handler(
    Json(claim_job_input): Json<ClaimJobInput>,
) -> Result<Json<ClaimJobResponse>, (StatusCode, String)> {
    let worker_id = claim_job_input.worker_id.trim();

    if worker_id.is_empty() {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            "worker_id cannot be empty.".into(),
        ));
    }

    let connection = &mut establish_connection();

    let result = connection.transaction::<Option<Job>, diesel::result::Error, _>(|connection| {
        let first_job = find_first_available_job(connection);

        if first_job.is_err() {
            return Ok(None);
        }

        let job = first_job.unwrap();

        let update_result = diesel::update(self::schema::jobs::dsl::jobs)
            .filter(schema::jobs::id.eq(job.id.clone()))
            .set((
                schema::jobs::status.eq("running"),
                schema::jobs::worker_id.eq(worker_id),
                schema::jobs::updated_at.eq(diesel::dsl::now),
            ))
            .execute(connection);

        if update_result.is_err() {
            return Err(update_result.unwrap_err());
        }

        let updated_job = find_job(connection, &job.id);

        if updated_job.is_err() {
            return Err(updated_job.unwrap_err());
        }

        let job = updated_job.unwrap();

        return diesel::result::QueryResult::Ok(Some(job));
    });

    if result.is_err() {
        log::error!("Failed to claim job. {}", result.unwrap_err());

        return Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to claim job.".into(),
        ));
    }

    let job = result.unwrap();

    match job {
        Some(job) => {
            log::info!(
                "Claiming job with id {:?} for worker {:?}",
                job.id,
                worker_id
            );

            return Ok(Json(ClaimJobResponse {
                job: Some(transform_job(job)),
            }));
        }
        None => {
            log::info!("There are no available jobs to claim.");

            return Ok(Json(ClaimJobResponse { job: None }));
        }
    }
}

#[derive(Deserialize)]
pub struct CreateSuccessfulRunInput {
    id: String,
}

#[derive(Serialize)]
pub struct CreateSuccessfulRunResponse {
    job: JobDto,
}

pub async fn create_successful_run_handler(
    Json(create_successful_run_input): Json<CreateSuccessfulRunInput>,
) -> Result<Json<CreateSuccessfulRunResponse>, (StatusCode, String)> {
    let id = create_successful_run_input.id.trim();

    let connection = &mut establish_connection();

    let existing_job = find_job(connection, id);

    if existing_job.is_err() {
        return Err((
            axum::http::StatusCode::NOT_FOUND,
            "Job does not exist.".into(),
        ));
    }

    let existing_job = existing_job.unwrap();

    if existing_job.status != "running" {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            "Job is not running.".into(),
        ));
    }

    let job = connection.transaction::<Job, diesel::result::Error, _>(|connection| {
        let update_result = diesel::update(self::schema::jobs::dsl::jobs)
            .filter(schema::jobs::id.eq(id))
            .set((
                schema::jobs::status.eq("success"),
                schema::jobs::updated_at.eq(diesel::dsl::now),
            ))
            .execute(connection);

        if update_result.is_err() {
            return Err(update_result.unwrap_err());
        }

        let job = find_job(connection, id);

        if job.is_err() {
            return Err(job.unwrap_err());
        }

        return Ok(job.unwrap());
    });

    match job {
        Err(e) => {
            log::error!("Failed to update job. {}", e);

            return Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to update job.".into(),
            ));
        }
        Ok(job) => {
            log::info!("Updated job with id {:?} to success.", job.id);

            return Ok(Json(CreateSuccessfulRunResponse {
                job: transform_job(job),
            }));
        }
    }
}

#[derive(Deserialize)]
pub struct CreateFailedRunInput {
    id: String,
    error: String,
}

#[derive(Serialize)]
pub struct CreateFailedRunResponse {
    job: JobDto,
}

pub async fn create_failed_run_handler(
    Json(create_failed_run_input): Json<CreateFailedRunInput>,
) -> Result<Json<CreateFailedRunResponse>, (StatusCode, String)> {
    let id = create_failed_run_input.id.trim();
    let error = create_failed_run_input.error.trim();

    let connection = &mut establish_connection();

    let existing_job = find_job(connection, id);

    if existing_job.is_err() {
        return Err((
            axum::http::StatusCode::NOT_FOUND,
            "Job does not exist.".into(),
        ));
    }

    let existing_job = existing_job.unwrap();

    if existing_job.status != "running" {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            "Job is not running.".into(),
        ));
    }

    let job = connection.transaction::<Job, diesel::result::Error, _>(|connection| {
        let update_result = diesel::update(self::schema::jobs::dsl::jobs)
            .filter(schema::jobs::id.eq(id))
            .set((
                schema::jobs::status.eq("failure"),
                schema::jobs::last_error.eq(error),
                schema::jobs::updated_at.eq(diesel::dsl::now),
            ))
            .execute(connection);

        if update_result.is_err() {
            return Err(update_result.unwrap_err());
        }

        let job = find_job(connection, id);

        if job.is_err() {
            return Err(job.unwrap_err());
        }

        return Ok(job.unwrap());
    });

    match job {
        Err(e) => {
            log::error!("Failed to update job. {}", e);

            return Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to update job.".into(),
            ));
        }
        Ok(job) => {
            log::info!("Updated job with id {:?} to success.", job.id);

            return Ok(Json(CreateFailedRunResponse {
                job: transform_job(job),
            }));
        }
    }
}

fn find_first_available_job(
    connection: &mut SqliteConnection,
) -> Result<Job, diesel::result::Error> {
    use self::schema::jobs::dsl::jobs;

    let job = jobs
        .select(Job::as_select())
        .filter(schema::jobs::status.eq("pending"))
        .filter(schema::jobs::worker_id.is_null())
        .order(schema::jobs::created_at.asc())
        .first(connection);

    return job;
}

fn find_job(connection: &mut SqliteConnection, id: &str) -> Result<Job, diesel::result::Error> {
    use self::schema::jobs::dsl::jobs;

    let job = jobs
        .select(Job::as_select())
        .filter(schema::jobs::id.eq(id))
        .first(connection);

    return job;
}
