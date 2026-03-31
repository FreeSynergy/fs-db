// DbRecord — trait for domain types that can be stored and retrieved via
// a `Repository<T>` + `DbEngine`.

use fs_error::FsError;

use crate::engine::DbRow;

/// Trait that domain types implement to participate in the `Repository<T>`
/// abstraction.
///
/// Implementors must describe their table schema via associated constants and
/// supply serialization/deserialization to/from [`DbRow`].
///
/// # Derivation
///
/// There is no derive macro yet — implement by hand for each entity.
///
/// # Example
///
/// ```rust,ignore
/// use fs_db::record::DbRecord;
/// use fs_db::engine::DbRow;
/// use fs_error::FsError;
/// use serde_json::Value;
///
/// struct User { id: Option<i64>, name: String, email: String }
///
/// impl DbRecord for User {
///     fn table_name() -> &'static str { "users" }
///     fn column_names() -> &'static [&'static str] { &["name", "email"] }
///     fn primary_key(&self) -> Option<i64> { self.id }
///     fn set_primary_key(&mut self, id: i64) { self.id = Some(id); }
///
///     fn to_values(&self) -> Vec<Value> {
///         vec![
///             Value::String(self.name.clone()),
///             Value::String(self.email.clone()),
///         ]
///     }
///
///     fn from_row(row: &DbRow) -> Result<Self, FsError> {
///         Ok(User {
///             id:    Some(row.get_i64("id")?),
///             name:  row.get_string("name")?,
///             email: row.get_string("email")?,
///         })
///     }
/// }
/// ```
pub trait DbRecord: Sized + Send + Sync + 'static {
    /// Name of the SQL table this record lives in.
    fn table_name() -> &'static str;

    /// Column names, **excluding** the primary key column `id`.
    ///
    /// The order must match [`to_values`](Self::to_values).
    fn column_names() -> &'static [&'static str];

    /// Return the primary key value, or `None` for unsaved (new) records.
    fn primary_key(&self) -> Option<i64>;

    /// Set the primary key after a successful INSERT.
    fn set_primary_key(&mut self, id: i64);

    /// Serialize column values in the same order as [`column_names`].
    fn to_values(&self) -> Vec<serde_json::Value>;

    /// Deserialize a record from a database row.
    ///
    /// # Errors
    ///
    /// Returns [`FsError`] if a required column is missing or has the wrong type.
    fn from_row(row: &DbRow) -> Result<Self, FsError>;
}

// ── DbRow helpers ─────────────────────────────────────────────────────────────

/// Helper methods for extracting typed values from a [`DbRow`].
pub trait DbRowExt {
    /// Look up a column index by name.
    ///
    /// # Errors
    ///
    /// Returns [`FsError`] when `name` is not found in the row.
    fn col_index(&self, name: &str) -> Result<usize, FsError>;

    /// Extract an `i64` value.
    ///
    /// # Errors
    ///
    /// Returns [`FsError`] when the column is missing or not an integer.
    fn get_i64(&self, name: &str) -> Result<i64, FsError>;

    /// Extract a `String` value.
    ///
    /// # Errors
    ///
    /// Returns [`FsError`] when the column is missing or not a string.
    fn get_string(&self, name: &str) -> Result<String, FsError>;

    /// Extract an optional `String` value — `NULL` maps to `None`.
    ///
    /// # Errors
    ///
    /// Returns [`FsError`] when the column is missing entirely.
    fn get_opt_string(&self, name: &str) -> Result<Option<String>, FsError>;

    /// Extract an optional `i64` value — `NULL` maps to `None`.
    ///
    /// # Errors
    ///
    /// Returns [`FsError`] when the column is missing entirely.
    fn get_opt_i64(&self, name: &str) -> Result<Option<i64>, FsError>;

    /// Extract a `bool` value (stored as integer 0/1).
    ///
    /// # Errors
    ///
    /// Returns [`FsError`] when the column is missing or not a boolean/integer.
    fn get_bool(&self, name: &str) -> Result<bool, FsError>;
}

impl DbRowExt for DbRow {
    fn col_index(&self, name: &str) -> Result<usize, FsError> {
        self.columns
            .iter()
            .position(|c| c == name)
            .ok_or_else(|| FsError::internal(format!("column '{name}' not found in row")))
    }

    fn get_i64(&self, name: &str) -> Result<i64, FsError> {
        let idx = self.col_index(name)?;
        self.values[idx]
            .as_i64()
            .ok_or_else(|| FsError::internal(format!("column '{name}' is not an integer")))
    }

    fn get_string(&self, name: &str) -> Result<String, FsError> {
        let idx = self.col_index(name)?;
        match &self.values[idx] {
            serde_json::Value::String(s) => Ok(s.clone()),
            serde_json::Value::Null => Err(FsError::internal(format!("column '{name}' is NULL"))),
            other => Ok(other.to_string()),
        }
    }

    fn get_opt_string(&self, name: &str) -> Result<Option<String>, FsError> {
        let idx = self.col_index(name)?;
        Ok(match &self.values[idx] {
            serde_json::Value::Null => None,
            serde_json::Value::String(s) => Some(s.clone()),
            other => Some(other.to_string()),
        })
    }

    fn get_opt_i64(&self, name: &str) -> Result<Option<i64>, FsError> {
        let idx = self.col_index(name)?;
        Ok(match &self.values[idx] {
            serde_json::Value::Null => None,
            v => v
                .as_i64()
                .map(Some)
                .ok_or_else(|| FsError::internal(format!("column '{name}' is not an integer")))?,
        })
    }

    fn get_bool(&self, name: &str) -> Result<bool, FsError> {
        let idx = self.col_index(name)?;
        match &self.values[idx] {
            serde_json::Value::Bool(b) => Ok(*b),
            serde_json::Value::Number(n) => Ok(n.as_i64().unwrap_or(0) != 0),
            other => Err(FsError::internal(format!(
                "column '{name}' is not a boolean (got {other})"
            ))),
        }
    }
}
