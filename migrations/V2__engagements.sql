ALTER TABLE statuses ADD COLUMN replies_count INT DEFAULT 0;
ALTER TABLE statuses ADD COLUMN reblogs_count INT DEFAULT 0;
ALTER TABLE statuses ADD COLUMN favourites_count INT DEFAULT 0;
ALTER TABLE statuses ADD COLUMN engagements_count INT GENERATED ALWAYS AS (replies_count + reblogs_count + favourites_count) VIRTUAL;
DELETE FROM status_refreshes;
