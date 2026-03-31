// Repository<T> — generic async CRUD abstraction over DbEngine.
//
// Design: Repository Pattern.
// Consumer code depends only on `Repository<T>` — the concrete engine
// (SQLite, Postgres, …) is injected via `DbEngine`.

use async_trait::async_trait;

use fs_error::FsError;

use crate::filter::Filter;
use crate::record::DbRecord;

// ── Repository<T> trait ───────────────────────────────────────────────────────

/// Generic repository for persisting and retrieving [`DbRecord`] values.
///
/// All database I/O is async and engine-agnostic — pass any [`DbEngine`]
/// implementation to the concrete [`EngineRepository`].
///
/// # Example
///
/// ```rust,ignore
/// use fs_db::repo::Repository;
/// use fs_db::filter::{Filter, Order};
///
/// async fn active_users(repo: &impl Repository<User>) -> Vec<User> {
///     let f = Filter::eq("status", "active")
///         .order_by("name", Order::Ascending)
///         .limit(50);
///     repo.find(f).await.unwrap()
/// }
/// ```
#[async_trait]
pub trait Repository<T: DbRecord>: Send + Sync {
    /// Find a record by its primary key.  Returns `None` when not found.
    ///
    /// # Errors
    /// Returns [`FsError`] on database failure.
    async fn find_by_id(&self, id: i64) -> Result<Option<T>, FsError>;

    /// Return all records, unfiltered.
    ///
    /// # Errors
    /// Returns [`FsError`] on database failure.
    async fn find_all(&self) -> Result<Vec<T>, FsError>;

    /// Return records matching `filter`.
    ///
    /// # Errors
    /// Returns [`FsError`] on database failure.
    async fn find(&self, filter: Filter<T>) -> Result<Vec<T>, FsError>;

    /// Count records matching `filter`.
    ///
    /// # Errors
    /// Returns [`FsError`] on database failure.
    async fn count(&self, filter: Filter<T>) -> Result<u64, FsError>;

    /// Return `true` if at least one record matches `filter`.
    ///
    /// # Errors
    /// Returns [`FsError`] on database failure.
    async fn exists(&self, filter: Filter<T>) -> Result<bool, FsError>;

    /// INSERT a new record or UPDATE an existing one (upsert by primary key).
    ///
    /// When `record.primary_key()` is `None` an INSERT is performed and the
    /// generated id is written back into the returned record.  When a primary
    /// key is present, an UPDATE is performed.
    ///
    /// # Errors
    /// Returns [`FsError`] on database failure.
    async fn save(&self, record: T) -> Result<T, FsError>;

    /// Delete the record with the given primary key.
    ///
    /// Returns `true` if a row was actually deleted, `false` if it did not exist.
    ///
    /// # Errors
    /// Returns [`FsError`] on database failure.
    async fn delete(&self, id: i64) -> Result<bool, FsError>;
}

// ── EngineRepository ──────────────────────────────────────────────────────────

/// A [`Repository<T>`] implementation that delegates to a [`DbEngine`].
///
/// Construct with [`EngineRepository::new`] and pass any `DbEngine`
/// implementation.
///
/// [`DbEngine`]: crate::engine::DbEngine
pub struct EngineRepository<'a, E> {
    engine: &'a E,
}

impl<'a, E> EngineRepository<'a, E> {
    /// Create a new repository backed by `engine`.
    pub fn new(engine: &'a E) -> Self {
        Self { engine }
    }
}

#[async_trait]
impl<T, E> Repository<T> for EngineRepository<'_, E>
where
    T: DbRecord,
    E: crate::engine::DbEngine,
{
    async fn find_by_id(&self, id: i64) -> Result<Option<T>, FsError> {
        let sql = format!("SELECT * FROM {} WHERE id = ?", T::table_name());
        let rows = self
            .engine
            .execute(&sql, vec![serde_json::Value::Number(id.into())])
            .await?;

        rows.rows.first().map(|r| T::from_row(r)).transpose()
    }

    async fn find_all(&self) -> Result<Vec<T>, FsError> {
        let sql = format!("SELECT * FROM {}", T::table_name());
        let rows = self.engine.execute(&sql, vec![]).await?;
        rows.rows.iter().map(|r| T::from_row(r)).collect()
    }

    async fn find(&self, filter: Filter<T>) -> Result<Vec<T>, FsError> {
        let (fragment, params) = filter.to_sql();
        let sql = format!("SELECT * FROM {} {}", T::table_name(), fragment);
        let rows = self.engine.execute(&sql, params).await?;
        rows.rows.iter().map(|r| T::from_row(r)).collect()
    }

    async fn count(&self, filter: Filter<T>) -> Result<u64, FsError> {
        let (fragment, params) = filter.to_sql();
        // Strip ORDER BY / LIMIT / OFFSET for COUNT — they are irrelevant.
        let where_only = strip_pagination(&fragment);
        let sql = format!(
            "SELECT COUNT(*) AS n FROM {} {}",
            T::table_name(),
            where_only
        );
        let rows = self.engine.execute(&sql, params).await?;
        let n = rows
            .rows
            .first()
            .and_then(|r| {
                r.columns
                    .iter()
                    .position(|c| c == "n")
                    .and_then(|i| r.values[i].as_u64())
            })
            .unwrap_or(0);
        Ok(n)
    }

    async fn exists(&self, filter: Filter<T>) -> Result<bool, FsError> {
        Ok(self.count(filter).await? > 0)
    }

    async fn save(&self, mut record: T) -> Result<T, FsError> {
        match record.primary_key() {
            None => {
                // INSERT
                let cols = T::column_names().join(", ");
                let placeholders = T::column_names()
                    .iter()
                    .map(|_| "?")
                    .collect::<Vec<_>>()
                    .join(", ");
                let sql = format!(
                    "INSERT INTO {} ({}) VALUES ({})",
                    T::table_name(),
                    cols,
                    placeholders
                );
                let params = record.to_values();
                let result = self.engine.execute(&sql, params).await?;
                // Retrieve the last inserted id.
                let id_rows = self
                    .engine
                    .execute("SELECT last_insert_rowid() AS id", vec![])
                    .await?;
                let id = id_rows
                    .rows
                    .first()
                    .and_then(|r| r.columns.iter().position(|c| c == "id"))
                    .and_then(|i| id_rows.rows[0].values[i].as_i64())
                    .ok_or_else(|| {
                        FsError::internal(format!(
                            "INSERT into {} produced no rowid (rows_affected={})",
                            T::table_name(),
                            result.rows_affected
                        ))
                    })?;
                record.set_primary_key(id);
                Ok(record)
            }
            Some(id) => {
                // UPDATE
                let set_clause = T::column_names()
                    .iter()
                    .map(|c| format!("{c} = ?"))
                    .collect::<Vec<_>>()
                    .join(", ");
                let sql = format!("UPDATE {} SET {} WHERE id = ?", T::table_name(), set_clause);
                let mut params = record.to_values();
                params.push(serde_json::Value::Number(id.into()));
                self.engine.execute(&sql, params).await?;
                Ok(record)
            }
        }
    }

    async fn delete(&self, id: i64) -> Result<bool, FsError> {
        let sql = format!("DELETE FROM {} WHERE id = ?", T::table_name());
        let result = self
            .engine
            .execute(&sql, vec![serde_json::Value::Number(id.into())])
            .await?;
        Ok(result.rows_affected > 0)
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Strip ORDER BY / LIMIT / OFFSET from a filter fragment (for COUNT queries).
fn strip_pagination(fragment: &str) -> &str {
    // Find the first occurrence of ORDER BY, LIMIT, or OFFSET and truncate there.
    for keyword in &[" ORDER BY ", " LIMIT ", " OFFSET "] {
        if let Some(pos) = fragment.to_uppercase().find(keyword) {
            return &fragment[..pos];
        }
    }
    fragment
}
