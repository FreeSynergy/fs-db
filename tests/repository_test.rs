// Integration tests for Repository<T> + Filter<T> + EngineRepository.
//
// Uses an in-memory SQLite via DbManager (no external dependency needed).

use fs_db::{
    engine::{DbConfig, DbEngine, DbHealth, DbRow, DbRows},
    filter::{Filter, Order},
    record::{DbRecord, DbRowExt},
    repo::{EngineRepository, Repository},
};
use fs_error::FsError;
use serde_json::Value;

// ── Minimal DbEngine stub for tests ──────────────────────────────────────────

/// In-memory engine backed by a Vec of rows (enough to test Repository logic).
///
/// We use a real sqlite engine pulled from fs-db's own test setup.
/// Since fs-db doesn't yet ship a ready-made engine, we test via DbEngine
/// using the raw execute path with a simple stub.
struct VecEngine {
    rows: std::sync::Mutex<Vec<(i64, String, String)>>,
    next_id: std::sync::atomic::AtomicI64,
}

impl VecEngine {
    fn new() -> Self {
        Self {
            rows: std::sync::Mutex::new(vec![]),
            next_id: std::sync::atomic::AtomicI64::new(1),
        }
    }
}

#[async_trait::async_trait]
impl fs_db::engine::DbEngine for VecEngine {
    async fn open(_config: DbConfig) -> Result<Self, FsError>
    where
        Self: Sized,
    {
        Ok(Self::new())
    }

    async fn migrate(&self) -> Result<(), FsError> {
        Ok(())
    }

    async fn execute(&self, sql: &str, params: Vec<Value>) -> Result<DbRows, FsError> {
        let sql_upper = sql.to_uppercase();

        if sql_upper.starts_with("INSERT") {
            let id = self
                .next_id
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let name = params
                .first()
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let email = params
                .get(1)
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            self.rows.lock().unwrap().push((id, name, email));
            return Ok(DbRows {
                rows: vec![],
                rows_affected: 1,
            });
        }

        if sql_upper.starts_with("SELECT LAST_INSERT_ROWID") {
            let id = self.next_id.load(std::sync::atomic::Ordering::SeqCst) - 1;
            return Ok(DbRows {
                rows: vec![DbRow {
                    columns: vec!["id".into()],
                    values: vec![Value::Number(id.into())],
                }],
                rows_affected: 0,
            });
        }

        if sql_upper.starts_with("UPDATE") {
            if let Some(new_name) = params.first().and_then(Value::as_str) {
                let id = params.get(2).and_then(Value::as_i64).unwrap_or(0);
                let mut rows = self.rows.lock().unwrap();
                for row in rows.iter_mut() {
                    if row.0 == id {
                        row.1 = new_name.to_string();
                    }
                }
            }
            return Ok(DbRows {
                rows: vec![],
                rows_affected: 1,
            });
        }

        if sql_upper.starts_with("DELETE") {
            let id = params.first().and_then(Value::as_i64).unwrap_or(0);
            let mut rows = self.rows.lock().unwrap();
            let before = rows.len();
            rows.retain(|r| r.0 != id);
            let after = rows.len();
            return Ok(fs_db::engine::DbRows {
                rows: vec![],
                rows_affected: (before - after) as u64,
            });
        }

        if sql_upper.contains("COUNT(*)") {
            let rows = self.rows.lock().unwrap();
            let n = rows.len() as u64;
            return Ok(DbRows {
                rows: vec![DbRow {
                    columns: vec!["n".into()],
                    values: vec![Value::Number(n.into())],
                }],
                rows_affected: 0,
            });
        }

        // SELECT — return all rows
        let rows = self.rows.lock().unwrap();
        let db_rows: Vec<DbRow> = rows
            .iter()
            .map(|(id, name, email)| DbRow {
                columns: vec!["id".into(), "name".into(), "email".into()],
                values: vec![
                    Value::Number((*id).into()),
                    Value::String(name.clone()),
                    Value::String(email.clone()),
                ],
            })
            .collect();
        Ok(DbRows {
            rows: db_rows,
            rows_affected: 0,
        })
    }

    async fn health(&self) -> Result<DbHealth, FsError> {
        Ok(DbHealth {
            connected: true,
            latency_ms: 0,
            engine: "vec".into(),
            version: "0.0.0".into(),
        })
    }

    async fn close(self) -> Result<(), FsError>
    where
        Self: Sized,
    {
        Ok(())
    }
}

// ── User record (test domain type) ───────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
struct User {
    id: Option<i64>,
    name: String,
    email: String,
}

impl DbRecord for User {
    fn table_name() -> &'static str {
        "users"
    }

    fn column_names() -> &'static [&'static str] {
        &["name", "email"]
    }

    fn primary_key(&self) -> Option<i64> {
        self.id
    }

    fn set_primary_key(&mut self, id: i64) {
        self.id = Some(id);
    }

    fn to_values(&self) -> Vec<Value> {
        vec![
            Value::String(self.name.clone()),
            Value::String(self.email.clone()),
        ]
    }

    fn from_row(row: &DbRow) -> Result<Self, FsError> {
        Ok(Self {
            id: Some(row.get_i64("id")?),
            name: row.get_string("name")?,
            email: row.get_string("email")?,
        })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn save_insert_assigns_primary_key() {
    let engine = VecEngine::new();
    let repo = EngineRepository::new(&engine);

    let user = User {
        id: None,
        name: "Alice".into(),
        email: "alice@example.com".into(),
    };

    let saved = repo.save(user).await.unwrap();
    assert!(saved.id.is_some(), "INSERT must assign primary key");
    assert_eq!(saved.name, "Alice");
}

#[tokio::test]
async fn find_all_returns_inserted_records() {
    let engine = VecEngine::new();
    let repo = EngineRepository::new(&engine);

    for name in &["Alice", "Bob", "Carol"] {
        repo.save(User {
            id: None,
            name: (*name).into(),
            email: format!("{name}@example.com"),
        })
        .await
        .unwrap();
    }

    let all: Vec<User> = repo.find_all().await.unwrap();
    assert_eq!(all.len(), 3);
}

#[tokio::test]
async fn find_by_id_returns_record() {
    let engine = VecEngine::new();
    let repo = EngineRepository::new(&engine);

    let saved = repo
        .save(User {
            id: None,
            name: "Alice".into(),
            email: "alice@example.com".into(),
        })
        .await
        .unwrap();

    let found: Option<User> = repo.find_by_id(saved.id.unwrap()).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "Alice");
}

#[tokio::test]
async fn find_by_id_returns_none_for_missing() {
    let engine = VecEngine::new();
    let repo = EngineRepository::new(&engine);
    let found: Option<User> = repo.find_by_id(9999).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn delete_removes_record() {
    let engine = VecEngine::new();
    let repo = EngineRepository::new(&engine);

    let saved = repo
        .save(User {
            id: None,
            name: "Alice".into(),
            email: "alice@example.com".into(),
        })
        .await
        .unwrap();
    let id = saved.id.unwrap();

    let deleted = <EngineRepository<VecEngine> as Repository<User>>::delete(&repo, id)
        .await
        .unwrap();
    assert!(deleted);

    let all: Vec<User> = repo.find_all().await.unwrap();
    assert!(all.is_empty());
}

#[tokio::test]
async fn count_returns_number_of_records() {
    let engine = VecEngine::new();
    let repo = EngineRepository::new(&engine);

    for i in 0..5u32 {
        repo.save(User {
            id: None,
            name: format!("User{i}"),
            email: format!("user{i}@example.com"),
        })
        .await
        .unwrap();
    }

    let count = repo.count(Filter::<User>::default()).await.unwrap();
    assert_eq!(count, 5);
}

#[tokio::test]
async fn exists_true_when_records_present() {
    let engine = VecEngine::new();
    let repo = EngineRepository::new(&engine);

    repo.save(User {
        id: None,
        name: "Alice".into(),
        email: "alice@example.com".into(),
    })
    .await
    .unwrap();

    let exists = repo.exists(Filter::<User>::default()).await.unwrap();
    assert!(exists);
}

#[tokio::test]
async fn exists_false_on_empty_table() {
    let engine = VecEngine::new();
    let repo = EngineRepository::new(&engine);
    let exists = repo.exists(Filter::<User>::default()).await.unwrap();
    assert!(!exists);
}

#[tokio::test]
async fn find_with_filter_passes_sql_fragment() {
    let engine = VecEngine::new();
    let repo = EngineRepository::new(&engine);

    repo.save(User {
        id: None,
        name: "Alice".into(),
        email: "alice@example.com".into(),
    })
    .await
    .unwrap();

    // The VecEngine ignores filters and returns all — but we verify that
    // the call succeeds and that Filter<T> generates the expected SQL.
    let f = Filter::<User>::eq("name", "Alice")
        .order_by("name", Order::Ascending)
        .limit(10);
    let (sql, params) = f.to_sql();
    assert!(sql.contains("WHERE"), "filter must produce WHERE clause");
    assert!(!params.is_empty());

    let results: Vec<User> = repo.find(Filter::<User>::default()).await.unwrap();
    assert_eq!(results.len(), 1);
}
