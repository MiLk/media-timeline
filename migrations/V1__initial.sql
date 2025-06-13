CREATE TABLE IF NOT EXISTS subscribed_hashtags(
    name TEXT NOT NULL PRIMARY KEY,
    approved INTEGER NOT NULL DEFAULT 0,
    votes INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (DATETIME('now'))
);
CREATE TABLE IF NOT EXISTS recent_statuses(
    tag TEXT NOT NULL PRIMARY KEY,
    status_id TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS statuses(
    id TEXT NOT NULL PRIMARY KEY,
    created_at TEXT NOT NULL,
    account_id TEXT NOT NULL,
    account_acct TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS status_tags(
    status_id TEXT NOT NULL,
    name TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS status_tags_idx ON status_tags (status_id, name);
CREATE TABLE IF NOT EXISTS status_refreshes(
    id TEXT NOT NULL PRIMARY KEY,
    refreshed_at TEXT NOT NULL
);
