-- Your SQL goes here
CREATE TABLE loading_statuses (
    id SERIAL PRIMARY KEY,
    description VARCHAR(16)
);

INSERT INTO loading_statuses (id, description) VALUES (0, 'In queue');
INSERT INTO loading_statuses (id, description) VALUES (1, 'In progress');
INSERT INTO loading_statuses (id, description) VALUES (2, 'Loaded');

ALTER TABLE signatures ADD COLUMN loading_status INTEGER REFERENCES loading_statuses(id);