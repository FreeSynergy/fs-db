/// Embedded SQL migration runner for `FreeSynergy`.
///
/// Migrations are SQL files in the `migrations/` directory, bundled into the
/// binary at compile time. They are applied in filename order and tracked in
/// a `_migrations` table so each runs exactly once.
///
/// # Design
///
/// The [`Migration`] trait is the abstract interface — implement it when you
/// need a custom migration runner.  [`Migrator`] is the built-in implementation
/// that reads embedded SQL files.
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};

use fs_error::FsError;

// ── Migration trait ───────────────────────────────────────────────────────────

/// Abstract migration interface.
///
/// Consumer code should depend on this trait, not on the concrete [`Migrator`].
/// Adapter crates (e.g. `fs-db-engine-sqlite`) may provide their own
/// implementation backed by the engine's native migration mechanism.
#[allow(async_fn_in_trait)]
pub trait Migration {
    /// Apply all pending migrations.
    ///
    /// Safe to call on every startup — already-applied migrations are skipped.
    ///
    /// # Errors
    ///
    /// Returns [`FsError`] if a migration fails.
    async fn apply_pending(&self) -> Result<(), FsError>;

    /// Roll back the most recently applied migration.
    ///
    /// Returns `true` if a migration was rolled back, `false` if none was
    /// applied yet.
    ///
    /// # Errors
    ///
    /// Returns [`FsError`] if the rollback fails or is not supported.
    async fn rollback_last(&self) -> Result<bool, FsError>;

    /// Return the name of the most recently applied migration, or `None` if no
    /// migrations have been applied.
    ///
    /// # Errors
    ///
    /// Returns [`FsError`] on database failure.
    async fn version(&self) -> Result<Option<String>, FsError>;
}

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

/// Applies embedded SQL migrations against a `SeaORM` database connection.
///
/// Implements the [`Migration`] trait.
///
/// # Example
///
/// ```rust,ignore
/// use fs_db::migration::{Migration, Migrator};
///
/// let runner = Migrator::new(&db);
/// runner.apply_pending().await?;
/// println!("at version: {:?}", runner.version().await?);
/// ```
pub struct Migrator<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> Migrator<'a> {
    /// Create a new [`Migrator`] backed by `db`.
    #[must_use]
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Convenience: run all pending migrations directly without constructing a
    /// `Migrator` instance.
    ///
    /// # Errors
    ///
    /// Returns [`FsError`] if a migration fails.
    pub async fn run(db: &'a DatabaseConnection) -> Result<(), FsError> {
        Self::new(db).apply_pending().await
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

    async fn do_apply(db: &DatabaseConnection, name: &str, sql: &str) -> Result<(), FsError> {
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

impl Migration for Migrator<'_> {
    async fn apply_pending(&self) -> Result<(), FsError> {
        Self::ensure_tracking_table(self.db).await?;
        for (name, sql) in MIGRATIONS {
            if Self::is_applied(self.db, name).await? {
                continue;
            }
            Self::do_apply(self.db, name, sql).await?;
        }
        Ok(())
    }

    async fn rollback_last(&self) -> Result<bool, FsError> {
        Self::ensure_tracking_table(self.db).await?;
        let last = self.version().await?;
        let Some(name) = last else {
            return Ok(false);
        };
        let sql = format!("DELETE FROM _migrations WHERE name = '{name}'");
        self.db
            .execute_unprepared(&sql)
            .await
            .map_err(|e| FsError::internal(format!("rollback: {e}")))?;
        Ok(true)
    }

    async fn version(&self) -> Result<Option<String>, FsError> {
        Self::ensure_tracking_table(self.db).await?;
        let sql =
            "SELECT name FROM _migrations ORDER BY applied_at DESC, name DESC LIMIT 1".to_owned();
        let result = self
            .db
            .query_one_raw(Statement::from_string(self.db.get_database_backend(), sql))
            .await
            .map_err(|e| FsError::internal(format!("migration version: {e}")))?;
        Ok(result.and_then(|row| row.try_get::<String>("", "name").ok()))
    }
}
