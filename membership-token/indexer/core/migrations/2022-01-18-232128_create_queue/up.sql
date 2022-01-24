-- Your SQL goes here
CREATE TABLE signatures (
    id SERIAL PRIMARY KEY,
    signature VARCHAR(88),
    slot INTEGER,
    err TEXT,
    memo TEXT,   
    block_time INTEGER,
    confirmation_status VARCHAR(16)
);

CREATE TABLE transactions (
    id SERIAL PRIMARY KEY,
    signature VARCHAR(88),
    slot INTEGER,
    transaction TEXT,
    block_time INTEGER
);