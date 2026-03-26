-- 003_installed_packages.sql
-- Tracks installed package versions with rollback support.
CREATE TABLE IF NOT EXISTS installed_packages (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    package_id     TEXT    NOT NULL,
    version        TEXT    NOT NULL,
    channel        TEXT    NOT NULL DEFAULT 'stable',
    package_type   TEXT    NOT NULL DEFAULT 'program',
    active         INTEGER NOT NULL DEFAULT 1,
    signature      TEXT,
    trust_unsigned INTEGER NOT NULL DEFAULT 0,
    installed_at   INTEGER NOT NULL,
    updated_at     INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_installed_packages_package_id
    ON installed_packages (package_id);

CREATE INDEX IF NOT EXISTS idx_installed_packages_active
    ON installed_packages (package_id, active);
