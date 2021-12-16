CREATE TABLE IF NOT EXISTS __B_users (
    id SERIAL PRIMARY KEY,

    username VARCHAR(1024) NOT NULL UNIQUE,
    password_hash VARCHAR(2048) NOT NULL,

    user_group VARCHAR(1024) NOT NULL
);
