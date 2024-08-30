// @generated automatically by Diesel CLI.

diesel::table! {
    jobs (id) {
        id -> Text,
        created_at -> Timestamp,
        last_error -> Nullable<Text>,
        payload -> Text,
        max_retries -> Integer,
        name -> Text,
        retry_count -> Integer,
        status -> Text,
        updated_at -> Nullable<Timestamp>,
        worker_id -> Nullable<Text>,
    }
}
