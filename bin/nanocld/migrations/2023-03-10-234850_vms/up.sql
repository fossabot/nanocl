-- Your SQL goes here
CREATE TABLE IF NOT EXISTS "vm_images" (
  "name" VARCHAR NOT NULL PRIMARY KEY,
  "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  "path" VARCHAR NOT NULL,
  "type" VARCHAR NOT NULL,
  "size" BIGINT NOT NULL,
  "checksum" VARCHAR NOT NULL
);

CREATE TABLE IF NOT EXISTS "vm_configs" (
  "key" UUID NOT NULL UNIQUE PRIMARY KEY,
  "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  "vm_key" VARCHAR NOT NULL,
  "version" VARCHAR NOT NULL,
  "config" JSON NOT NULL
);

CREATE TABLE IF NOT EXISTS "vms" (
  "key" VARCHAR NOT NULL UNIQUE PRIMARY KEY,
  "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  "name" VARCHAR NOT NULL,
  "config_key" UUID NOT NULL REFERENCES vm_configs("key"),
  "namespace_name" VARCHAR NOT NULL REFERENCES namespaces("name")
);
