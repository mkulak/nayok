CREATE TABLE events(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at TIMESTAMP default CURRENT_TIMESTAMP,
    relative_uri TEXT,
    method TEXT,
    headers TEXT
--     body BLOB
);

CREATE INDEX events_create_date_idx ON events(created_at);