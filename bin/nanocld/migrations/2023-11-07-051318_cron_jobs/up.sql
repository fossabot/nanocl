-- Your SQL goes here
CREATE TABLE IF NOT EXISTS "cron_jobs" (
  "key" VARCHAR NOT NULL UNIQUE PRIMARY KEY,
  "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  "updated_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  "data" JSON NOT NULL,
  "metadata" JSON
);
