-- D1 Database Schema for code-llm-test-api
-- Run with: wrangler d1 execute rank-data --file=api/schema.sql

CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    github_id INTEGER UNIQUE NOT NULL,
    login TEXT NOT NULL,
    avatar_url TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_github_id ON users(github_id);

CREATE TABLE IF NOT EXISTS websites (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    domains TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS models (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    aliases TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS website_models (
    website_id TEXT NOT NULL REFERENCES websites(id),
    model_id TEXT NOT NULL REFERENCES models(id),
    model_weight REAL NOT NULL DEFAULT 1.0,
    website_weight REAL NOT NULL DEFAULT 1.0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (website_id, model_id)
);
CREATE INDEX IF NOT EXISTS idx_wm_model ON website_models(model_id);

CREATE TABLE IF NOT EXISTS submissions (
    id TEXT PRIMARY KEY,
    submitter_type TEXT NOT NULL CHECK(submitter_type IN ('oauth','anonymous')),
    user_id TEXT,
    ip_hash TEXT,
    website_id TEXT NOT NULL REFERENCES websites(id),
    model_id TEXT NOT NULL REFERENCES models(id),
    test_suite_version TEXT NOT NULL,
    api_style TEXT NOT NULL,
    endpoint_hash TEXT NOT NULL,
    total_score REAL NOT NULL,
    dimension_scores TEXT NOT NULL,
    test_results TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_oauth_uniq ON submissions(user_id, website_id, model_id) WHERE submitter_type = 'oauth';
CREATE UNIQUE INDEX IF NOT EXISTS idx_anon_uniq ON submissions(ip_hash, website_id, model_id) WHERE submitter_type = 'anonymous';
CREATE INDEX IF NOT EXISTS idx_sub_website ON submissions(website_id);
CREATE INDEX IF NOT EXISTS idx_sub_model ON submissions(model_id);
CREATE INDEX IF NOT EXISTS idx_sub_ip_created ON submissions(ip_hash, created_at);
