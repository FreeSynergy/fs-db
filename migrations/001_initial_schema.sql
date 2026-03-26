-- Migration 001 — Initial FreeSynergy core schema
-- Supports SQLite (default) and PostgreSQL.
-- Run this file once against a fresh database.

-- ── resources ─────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS resources (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    kind        TEXT    NOT NULL,
    name        TEXT    NOT NULL,
    project_id  INTEGER REFERENCES resources(id) ON DELETE SET NULL,
    parent_id   INTEGER REFERENCES resources(id) ON DELETE CASCADE,
    meta        TEXT,
    created_at  INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at  INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_resources_kind       ON resources(kind);
CREATE INDEX IF NOT EXISTS idx_resources_project_id ON resources(project_id);

-- ── permissions ───────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS permissions (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    subject     TEXT    NOT NULL,
    action      TEXT    NOT NULL,
    resource_id INTEGER REFERENCES resources(id) ON DELETE CASCADE,
    granted_at  INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    expires_at  INTEGER
);

CREATE INDEX IF NOT EXISTS idx_permissions_subject ON permissions(subject);
CREATE INDEX IF NOT EXISTS idx_permissions_action  ON permissions(action);

-- ── sync_states ───────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS sync_states (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    resource_id   INTEGER NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    node_id       TEXT    NOT NULL,
    vector_clock  TEXT    NOT NULL DEFAULT '{}',
    pending_ops   TEXT,
    last_synced   INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    UNIQUE (resource_id, node_id)
);

CREATE INDEX IF NOT EXISTS idx_sync_states_resource_id ON sync_states(resource_id);

-- ── plugins ───────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS plugins (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    name         TEXT    NOT NULL UNIQUE,
    version      TEXT    NOT NULL,
    kind         TEXT    NOT NULL DEFAULT 'wasm',
    wasm_hash    TEXT,
    path         TEXT,
    enabled      INTEGER NOT NULL DEFAULT 1,
    meta         TEXT,
    installed_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- ── audit_logs ────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS audit_logs (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    actor         TEXT    NOT NULL,
    action        TEXT    NOT NULL,
    resource_id   INTEGER REFERENCES resources(id) ON DELETE SET NULL,
    resource_kind TEXT,
    payload       TEXT,
    source        TEXT,
    outcome       TEXT    NOT NULL DEFAULT 'ok',
    created_at    INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_audit_logs_actor      ON audit_logs(actor);
CREATE INDEX IF NOT EXISTS idx_audit_logs_resource_id ON audit_logs(resource_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_created_at  ON audit_logs(created_at);
