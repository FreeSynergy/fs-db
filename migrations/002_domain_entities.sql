-- Migration 002 — Domain entities: hosts, projects, modules, service_registry
-- SQLite-compatible. Run after 001_initial_schema.

-- ── projects ──────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS projects (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL UNIQUE,
    domain      TEXT,
    description TEXT,
    status      TEXT    NOT NULL DEFAULT 'draft',
    created_at  INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at  INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_projects_status ON projects(status);

-- ── hosts ─────────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS hosts (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    name          TEXT    NOT NULL,
    fqdn          TEXT    NOT NULL,
    ip_address    TEXT    NOT NULL,
    ssh_port      INTEGER NOT NULL DEFAULT 22,
    status        TEXT    NOT NULL DEFAULT 'unknown',
    os            TEXT,
    architecture  TEXT,
    agent_version TEXT,
    project_id    INTEGER REFERENCES projects(id) ON DELETE SET NULL,
    joined_at     INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at    INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_hosts_status     ON hosts(status);
CREATE INDEX IF NOT EXISTS idx_hosts_project_id ON hosts(project_id);
CREATE INDEX IF NOT EXISTS idx_hosts_fqdn       ON hosts(fqdn);

-- ── modules ───────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS modules (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL,
    module_type TEXT    NOT NULL,
    host_id     INTEGER NOT NULL REFERENCES hosts(id) ON DELETE CASCADE,
    project_id  INTEGER REFERENCES projects(id) ON DELETE SET NULL,
    status      TEXT    NOT NULL DEFAULT 'stopped',
    version     TEXT,
    config      TEXT,
    created_at  INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at  INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_modules_host_id     ON modules(host_id);
CREATE INDEX IF NOT EXISTS idx_modules_project_id  ON modules(project_id);
CREATE INDEX IF NOT EXISTS idx_modules_status      ON modules(status);
CREATE INDEX IF NOT EXISTS idx_modules_module_type ON modules(module_type);

-- ── service_registry ──────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS service_registry (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    module_id    INTEGER NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
    module_name  TEXT    NOT NULL,
    capabilities TEXT    NOT NULL DEFAULT '[]',
    endpoint_url TEXT,
    healthy      INTEGER NOT NULL DEFAULT 0,
    last_check   INTEGER
);

CREATE INDEX IF NOT EXISTS idx_service_registry_module_id ON service_registry(module_id);
CREATE INDEX IF NOT EXISTS idx_service_registry_healthy   ON service_registry(healthy);
