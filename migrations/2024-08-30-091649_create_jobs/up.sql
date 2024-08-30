CREATE TABLE jobs (
    id TEXT PRIMARY KEY NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_error TEXT,
    payload TEXT NOT NULL,
    max_retries INTEGER NOT NULL DEFAULT 3,
    name TEXT NOT NULL,
    retry_count INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'pending',
    updated_at TIMESTAMP
);