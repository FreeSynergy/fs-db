// Abstract database engine trait for FreeSynergy.
//
// Define once here — implementations live in separate adapter crates
// (fs-db-engine-sqlite, fs-db-engine-postgres, …).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use fs_error::FsError;

// ── Configuration ─────────────────────────────────────────────────────────────

/// Connection configuration passed to [`DbEngine::open`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConfig {
    /// Database URL.
    ///
    /// - file:    `sqlite:///path/to/db.sqlite3`
    /// - memory:  `sqlite::memory:`
    /// - postgres: `postgres://user:pass@host:5432/db`
    pub url: String,

    /// Maximum connections in the pool (default: 5).
    pub max_connections: u32,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            url: "sqlite::memory:".into(),
            max_connections: 5,
        }
    }
}

impl DbConfig {
    /// Create a config for a sqlite file path.
    #[must_use]
    pub fn sqlite(path: impl Into<String>) -> Self {
        let path = path.into();
        let url = if path == ":memory:" {
            "sqlite::memory:".into()
        } else {
            format!("sqlite://{path}?mode=rwc")
        };
        Self {
            url,
            max_connections: 5,
        }
    }
}

// ── Result types ─────────────────────────────────────────────────────────────

/// A single row returned by [`DbEngine::execute`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbRow {
    /// Column names in order.
    pub columns: Vec<String>,
    /// Column values as JSON.  `null` for SQL NULL.
    pub values: Vec<serde_json::Value>,
}

/// Result set returned by [`DbEngine::execute`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbRows {
    /// Rows returned by a SELECT or similar query.
    pub rows: Vec<DbRow>,
    /// Number of rows inserted / updated / deleted (for DML).
    pub rows_affected: u64,
}

/// Health status of a running engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbHealth {
    /// `true` if the connection pool is alive.
    pub connected: bool,
    /// Round-trip latency measured by a simple probe query.
    pub latency_ms: u64,
    /// Engine backend name (e.g. `"sqlite"`, `"postgres"`).
    pub engine: String,
    /// Database server / library version string.
    pub version: String,
}

// ── Trait ─────────────────────────────────────────────────────────────────────

/// Abstract database engine.
///
/// Implement this trait in an adapter crate to plug in a concrete backend.
/// Consumer code only ever depends on this trait — never on the concrete type.
///
/// # Example (consumer)
/// ```rust,ignore
/// use fs_db::engine::{DbConfig, DbEngine};
///
/// async fn run(engine: &impl DbEngine) {
///     engine.migrate().await.unwrap();
///     let rows = engine.execute("SELECT 1 AS n", vec![]).await.unwrap();
///     println!("{} row(s)", rows.rows.len());
/// }
/// ```
#[async_trait]
pub trait DbEngine: Send + Sync {
    /// Open a connection pool using the given [`DbConfig`].
    ///
    /// # Errors
    /// Returns [`FsError`] if the connection cannot be established.
    async fn open(config: DbConfig) -> Result<Self, FsError>
    where
        Self: Sized;

    /// Apply all pending schema migrations.
    ///
    /// # Errors
    /// Returns [`FsError`] if a migration fails.
    async fn migrate(&self) -> Result<(), FsError>;

    /// Execute `sql` with positional JSON `params` and return a result set.
    ///
    /// Both DML (`INSERT`, `UPDATE`, `DELETE`) and queries (`SELECT`) are
    /// supported.  For DML, [`DbRows::rows`] will be empty and
    /// [`DbRows::rows_affected`] will reflect the number of changed rows.
    ///
    /// # Errors
    /// Returns [`FsError`] on any SQL or transport error.
    async fn execute(&self, sql: &str, params: Vec<serde_json::Value>) -> Result<DbRows, FsError>;

    /// Probe the connection and return health metadata.
    ///
    /// # Errors
    /// Returns [`FsError`] if the health check itself fails (distinct from
    /// `connected: false`, which is a valid health response).
    async fn health(&self) -> Result<DbHealth, FsError>;

    /// Gracefully shut down the connection pool.
    ///
    /// # Errors
    /// Returns [`FsError`] if the pool cannot be drained cleanly.
    async fn close(self) -> Result<(), FsError>
    where
        Self: Sized;
}
