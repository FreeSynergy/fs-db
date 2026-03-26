// Database connection management via SeaORM.

use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection};

use fs_error::FsError;

// ── DbBackend ─────────────────────────────────────────────────────────────────

/// Database backend selection.
#[derive(Debug, Clone)]
pub enum DbBackend {
    /// SQLite file database.
    Sqlite { path: String },
    /// In-memory SQLite (for tests).
    SqliteMemory,
    /// PostgreSQL.
    #[cfg(feature = "postgres")]
    Postgres { url: String },
}

impl DbBackend {
    /// Build the SeaORM connection URL.
    pub fn url(&self) -> String {
        match self {
            DbBackend::Sqlite { path } => format!("sqlite://{path}?mode=rwc"),
            DbBackend::SqliteMemory => "sqlite::memory:".to_string(),
            #[cfg(feature = "postgres")]
            DbBackend::Postgres { url } => url.clone(),
        }
    }
}

// ── DbConnection ──────────────────────────────────────────────────────────────

/// SeaORM database connection wrapper.
pub struct DbConnection {
    conn: DatabaseConnection,
    backend: DbBackend,
}

impl DbConnection {
    /// Connect to the given backend with default options.
    pub async fn connect(backend: DbBackend) -> Result<Self, FsError> {
        let conn = Database::connect(backend.url())
            .await
            .map_err(|e| FsError::internal(format!("db connect: {e}")))?;
        Ok(Self { conn, backend })
    }

    /// Connect with custom [`ConnectOptions`] (pool size, timeouts, etc.)
    pub async fn connect_with_options(
        backend: DbBackend,
        mut opts: ConnectOptions,
    ) -> Result<Self, FsError> {
        opts.sqlx_logging(false);
        let conn = Database::connect(opts)
            .await
            .map_err(|e| FsError::internal(format!("db connect: {e}")))?;
        Ok(Self { conn, backend })
    }

    /// Access the underlying SeaORM [`DatabaseConnection`].
    pub fn inner(&self) -> &DatabaseConnection {
        &self.conn
    }

    /// The backend this connection was opened against.
    pub fn backend(&self) -> &DbBackend {
        &self.backend
    }

    /// Execute all statements in a schema string, creating tables if they don't exist.
    ///
    /// Safe to call every startup — designed for idempotent `CREATE TABLE IF NOT EXISTS` schemas.
    pub async fn apply_schema(&self, schema: &str) -> Result<(), FsError> {
        for stmt in schema.split(';').map(str::trim).filter(|s| !s.is_empty()) {
            self.conn
                .execute_unprepared(stmt)
                .await
                .map_err(|e| FsError::internal(format!("schema apply: {e}")))?;
        }
        Ok(())
    }

    /// Close the connection pool.
    pub async fn close(self) -> Result<(), FsError> {
        self.conn
            .close()
            .await
            .map_err(|e| FsError::internal(format!("db close: {e}")))
    }
}
