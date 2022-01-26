-- This file should undo anything in `up.sql`
ALTER TABLE signatures DROP COLUMN loading_status;

DROP TABLE loading_statuses;