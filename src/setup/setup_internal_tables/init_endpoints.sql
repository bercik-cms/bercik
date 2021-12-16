CREATE TABLE IF NOT EXISTS __B_endpoints (
    id SERIAL PRIMARY KEY,
    
    req_path VARCHAR(2048) NOT NULL,
    req_method VARCHAR(50) NOT NULL,

    handler_info TEXT NOT NULL,
    allowed_groups TEXT NOT NULL
);
