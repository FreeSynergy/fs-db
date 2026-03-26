/// Embedded SQL migration runner for `FreeSynergy`.
///
/// Migrations are SQL files in the `migrations/` directory, bundled into the
/// binary at compile time. They are applied in filename order and tracked in
/// a `_migrations` table so each runs exactly once.
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};

use fs_error::FsError;

// ── Embedded migrations ───────────────────────────────────────────────────────

/// All migrations in order. Each entry is `(name, sql)`.
const MIGRATIONS: &[(&str, &str)] = &[
    (
        "001_initial_schema",
        include_str!("../migrations/001_initial_schema.sql"),
    ),
    (
        "002_domain_entities",
        include_str!("../migrations/002_domain_entities.sql"),
    ),
    (
        "003_installed_packages",
        include_str!("../migrations/003_installed_packages.sql"),
    ),
];

// ── Migrator ──────────────────────────────────────────────────────────────────

/// Applies pending database migrations.
///
/// Creates a `_migrations` tracking table on first run, then executes any
/// migration that has not yet been recorded.
pub struct Migrator;

impl Migrator {
    /// Run all pending migrations against `db`.
    ///
    /// Safe to call on every startup — already-applied migrations are skipped.
    ///
    /// # Errors
    ///
    /// Returns [`FsError`] if a migration fails or the tracking table cannot be created.
    pub async fn run(db: &DatabaseConnection) -> Result<(), FsError> {
        Self::ensure_tracking_table(db).await?;

        for (name, sql) in MIGRATIONS {
            if Self::is_applied(db, name).await? {
                continue;
            }
            Self::apply(db, name, sql).await?;
        }

        Ok(())
    }

    // ── private ───────────────────────────────────────────────────────────────

    async fn ensure_tracking_table(db: &DatabaseConnection) -> Result<(), FsError> {
        let sql = "CREATE TABLE IF NOT EXISTS _migrations (\
            name TEXT PRIMARY KEY, \
            applied_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))\
        )";
        db.execute_unprepared(sql)
            .await
            .map_err(|e| FsError::internal(format!("migration tracking table: {e}")))?;
        Ok(())
    }

    async fn is_applied(db: &DatabaseConnection, name: &str) -> Result<bool, FsError> {
        let sql = format!("SELECT COUNT(*) FROM _migrations WHERE name = '{name}'");
        let result = db
            .query_one_raw(Statement::from_string(db.get_database_backend(), sql))
            .await
            .map_err(|e| FsError::internal(format!("migration check: {e}")))?;

        Ok(result.is_some_and(|row| row.try_get::<i64>("", "COUNT(*)").unwrap_or(0) > 0))
    }

    async fn apply(db: &DatabaseConnection, name: &str, sql: &str) -> Result<(), FsError> {
        // Execute each statement in the migration file individually.
        for stmt in sql.split(';').map(str::trim).filter(|s| !s.is_empty()) {
            db.execute_unprepared(stmt)
                .await
                .map_err(|e| FsError::internal(format!("migration '{name}' failed: {e}")))?;
        }

        let record = format!("INSERT INTO _migrations (name) VALUES ('{name}')");
        db.execute_unprepared(&record)
            .await
            .map_err(|e| FsError::internal(format!("migration record '{name}': {e}")))?;

        Ok(())
    }
}
