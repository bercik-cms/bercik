CREATE TABLE IF NOT EXISTS __B_endpoints (
    id SERIAL PRIMARY KEY,

    req_path VARCHAR(1024) NOT NULL,
    req_method VARCHAR(50) NOT NULL,

    handler_info JSON NOT NULL
);
