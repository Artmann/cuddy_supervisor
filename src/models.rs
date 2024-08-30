// src/models.rs

use crate::schema::jobs;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable)]
#[diesel(table_name = jobs)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]

pub struct Job {
    pub id: String,
    pub created_at: NaiveDateTime,
    pub last_error: Option<String>,
    pub max_retries: i32,
    pub name: String,
    pub payload: String,
    pub retry_count: i32,
    pub status: String,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Insertable, Deserialize, Serialize)]
#[diesel(table_name = jobs)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewJob<'a> {
    pub id: &'a str,
    pub payload: &'a str,
    pub max_retries: i32,
    pub name: &'a str,
    pub retry_count: i32,
    pub status: String,
}
