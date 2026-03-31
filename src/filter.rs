// Type-safe query filter for FreeSynergy repositories.
#![allow(dead_code)] // Condition::And is only used in combined filters built at runtime
                     //
                     // Build filters with the constructor methods (`eq`, `ne`, …) and combine
                     // them with `and` / `or`.  Call `to_sql` to produce a
                     // `(WHERE … ORDER BY … LIMIT … OFFSET …, params)` pair suitable for
                     // `DbEngine::execute`.

use std::fmt::Write as _;
use std::marker::PhantomData;

use serde_json::Value;

// ── Internal condition representation ────────────────────────────────────────

#[derive(Debug, Clone)]
enum Condition {
    Eq { field: String, value: Value },
    Ne { field: String, value: Value },
    Gt { field: String, value: Value },
    Lt { field: String, value: Value },
    Gte { field: String, value: Value },
    Lte { field: String, value: Value },
    InList { field: String, values: Vec<Value> },
    Like { field: String, pattern: String },
    IsNull { field: String },
    And(Vec<Condition>),
    Or(Vec<Condition>),
}

// ── Sort direction ────────────────────────────────────────────────────────────

/// Sort direction used with [`Filter::order_by`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Order {
    /// Ascending sort — `ORDER BY field ASC`.
    Ascending,
    /// Descending sort — `ORDER BY field DESC`.
    Descending,
}

// ── Filter<T> ─────────────────────────────────────────────────────────────────

/// Type-safe query filter for [`Repository<T>`](crate::repository::Repository).
///
/// The type parameter `T` is a phantom marker that ties the filter to a
/// specific [`DbRecord`](crate::record::DbRecord) type.  All SQL generation is
/// done through [`to_sql`](Self::to_sql) which returns a
/// `(WHERE … ORDER BY … LIMIT … OFFSET …, params)` tuple.
///
/// # Example
///
/// ```rust,ignore
/// use fs_db::filter::{Filter, Order};
///
/// let f = Filter::<User>::eq("email", "alice@example.com")
///     .and(Filter::ne("status", "deleted"))
///     .order_by("created_at", Order::Descending)
///     .limit(10);
///
/// let (sql, params) = f.to_sql();
/// // sql  => "WHERE (email = ? AND status != ?) ORDER BY created_at DESC LIMIT 10"
/// ```
#[derive(Debug, Clone)]
pub struct Filter<T> {
    conditions: Vec<Condition>,
    order: Vec<(String, Order)>,
    limit: Option<u64>,
    offset: u64,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Default for Filter<T> {
    fn default() -> Self {
        Self {
            conditions: Vec::new(),
            order: Vec::new(),
            limit: None,
            offset: 0,
            _marker: PhantomData,
        }
    }
}

impl<T> Filter<T> {
    // ── Constructors ──────────────────────────────────────────────────────────

    /// Match rows where `field = value`.
    #[must_use]
    pub fn eq(field: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::single(Condition::Eq {
            field: field.into(),
            value: value.into(),
        })
    }

    /// Match rows where `field != value`.
    #[must_use]
    pub fn ne(field: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::single(Condition::Ne {
            field: field.into(),
            value: value.into(),
        })
    }

    /// Match rows where `field > value`.
    #[must_use]
    pub fn gt(field: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::single(Condition::Gt {
            field: field.into(),
            value: value.into(),
        })
    }

    /// Match rows where `field < value`.
    #[must_use]
    pub fn lt(field: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::single(Condition::Lt {
            field: field.into(),
            value: value.into(),
        })
    }

    /// Match rows where `field >= value`.
    #[must_use]
    pub fn gte(field: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::single(Condition::Gte {
            field: field.into(),
            value: value.into(),
        })
    }

    /// Match rows where `field <= value`.
    #[must_use]
    pub fn lte(field: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::single(Condition::Lte {
            field: field.into(),
            value: value.into(),
        })
    }

    /// Match rows where `field IN (values…)`.
    #[must_use]
    pub fn in_list(field: impl Into<String>, values: Vec<impl Into<Value>>) -> Self {
        Self::single(Condition::InList {
            field: field.into(),
            values: values.into_iter().map(Into::into).collect(),
        })
    }

    /// Match rows where `field LIKE pattern` (SQL `%` / `_` wildcards apply).
    #[must_use]
    pub fn like(field: impl Into<String>, pattern: impl Into<String>) -> Self {
        Self::single(Condition::Like {
            field: field.into(),
            pattern: pattern.into(),
        })
    }

    /// Match rows where `field IS NULL`.
    #[must_use]
    pub fn is_null(field: impl Into<String>) -> Self {
        Self::single(Condition::IsNull {
            field: field.into(),
        })
    }

    // ── Combinators ───────────────────────────────────────────────────────────

    /// AND-combine this filter with `other`.
    ///
    /// All conditions from both filters are merged at the top level and joined
    /// with AND.
    #[must_use]
    pub fn and(mut self, other: Self) -> Self {
        self.conditions.extend(other.conditions);
        // Merge ordering/pagination from `other` if not already set on `self`.
        if self.order.is_empty() {
            self.order = other.order;
        }
        if self.limit.is_none() {
            self.limit = other.limit;
        }
        if self.offset == 0 {
            self.offset = other.offset;
        }
        self
    }

    /// OR-combine this filter with `other`.
    ///
    /// Both sets of conditions are wrapped in a single OR group:
    /// `(cond_self OR cond_other)`.
    #[must_use]
    pub fn or(self, other: Self) -> Self {
        let combined = [self.conditions, other.conditions].concat();
        Self::single(Condition::Or(combined))
    }

    // ── Pagination / ordering ─────────────────────────────────────────────────

    /// Append a sort column.  Multiple calls add secondary sort keys.
    #[must_use]
    pub fn order_by(mut self, field: impl Into<String>, dir: Order) -> Self {
        self.order.push((field.into(), dir));
        self
    }

    /// Limit result set to at most `n` rows.
    #[must_use]
    pub fn limit(mut self, n: u64) -> Self {
        self.limit = Some(n);
        self
    }

    /// Skip the first `n` rows before returning results.
    #[must_use]
    pub fn offset(mut self, n: u64) -> Self {
        self.offset = n;
        self
    }

    // ── SQL generation ────────────────────────────────────────────────────────

    /// Produce a `(sql_fragment, params)` pair.
    ///
    /// `sql_fragment` starts with `WHERE` if there are conditions, otherwise it
    /// is empty.  `ORDER BY`, `LIMIT`, and `OFFSET` clauses are appended as
    /// needed.  All values are returned as positional `?` parameters.
    #[must_use]
    pub fn to_sql(&self) -> (String, Vec<Value>) {
        let mut params = Vec::new();
        let where_body = conditions_to_sql(&self.conditions, &mut params);

        let mut sql = if where_body.is_empty() {
            String::new()
        } else {
            format!("WHERE {where_body}")
        };

        if !self.order.is_empty() {
            let parts: Vec<String> = self
                .order
                .iter()
                .map(|(f, d)| {
                    let dir = if *d == Order::Ascending {
                        "ASC"
                    } else {
                        "DESC"
                    };
                    format!("{f} {dir}")
                })
                .collect();
            let _ = write!(sql, " ORDER BY {}", parts.join(", "));
        }

        if let Some(n) = self.limit {
            let _ = write!(sql, " LIMIT {n}");
        }

        if self.offset > 0 {
            let _ = write!(sql, " OFFSET {}", self.offset);
        }

        (sql, params)
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    fn single(c: Condition) -> Self {
        Self {
            conditions: vec![c],
            ..Self::default()
        }
    }
}

// ── Free functions (SQL building) ─────────────────────────────────────────────

fn conditions_to_sql(conditions: &[Condition], params: &mut Vec<Value>) -> String {
    conditions
        .iter()
        .map(|c| condition_to_sql(c, params))
        .collect::<Vec<_>>()
        .join(" AND ")
}

fn condition_to_sql(c: &Condition, params: &mut Vec<Value>) -> String {
    match c {
        Condition::Eq { field, value } => {
            params.push(value.clone());
            format!("{field} = ?")
        }
        Condition::Ne { field, value } => {
            params.push(value.clone());
            format!("{field} != ?")
        }
        Condition::Gt { field, value } => {
            params.push(value.clone());
            format!("{field} > ?")
        }
        Condition::Lt { field, value } => {
            params.push(value.clone());
            format!("{field} < ?")
        }
        Condition::Gte { field, value } => {
            params.push(value.clone());
            format!("{field} >= ?")
        }
        Condition::Lte { field, value } => {
            params.push(value.clone());
            format!("{field} <= ?")
        }
        Condition::InList { field, values } => {
            let placeholders = values.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
            params.extend(values.iter().cloned());
            format!("{field} IN ({placeholders})")
        }
        Condition::Like { field, pattern } => {
            params.push(Value::String(pattern.clone()));
            format!("{field} LIKE ?")
        }
        Condition::IsNull { field } => {
            format!("{field} IS NULL")
        }
        Condition::And(inner) => {
            let sql = inner
                .iter()
                .map(|ic| condition_to_sql(ic, params))
                .collect::<Vec<_>>()
                .join(" AND ");
            format!("({sql})")
        }
        Condition::Or(inner) => {
            let sql = inner
                .iter()
                .map(|ic| condition_to_sql(ic, params))
                .collect::<Vec<_>>()
                .join(" OR ");
            format!("({sql})")
        }
    }
}
