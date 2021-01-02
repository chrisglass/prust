CREATE TABLE IF NOT EXISTS paste (
    uuid UUID PRIMARY KEY,
    author TEXT NOT NULL,
    content TEXT NOT NULL,
    created TEXT NOT NULL
);