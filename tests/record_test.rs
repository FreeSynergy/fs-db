// Unit tests for DbRecord trait helpers (DbRowExt).

use fs_db::engine::{DbRow, DbRows};
use fs_db::record::{DbRecord, DbRowExt};

// ── Test record ───────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
struct Note {
    id: Option<i64>,
    title: String,
    body: Option<String>,
    pinned: bool,
}

impl DbRecord for Note {
    fn table_name() -> &'static str {
        "notes"
    }

    fn column_names() -> &'static [&'static str] {
        &["title", "body", "pinned"]
    }

    fn primary_key(&self) -> Option<i64> {
        self.id
    }

    fn set_primary_key(&mut self, id: i64) {
        self.id = Some(id);
    }

    fn to_values(&self) -> Vec<serde_json::Value> {
        vec![
            serde_json::Value::String(self.title.clone()),
            self.body
                .as_deref()
                .map(|s| serde_json::Value::String(s.to_owned()))
                .unwrap_or(serde_json::Value::Null),
            serde_json::Value::Bool(self.pinned),
        ]
    }

    fn from_row(row: &DbRow) -> Result<Self, fs_error::FsError> {
        Ok(Note {
            id: row.get_opt_i64("id")?,
            title: row.get_string("title")?,
            body: row.get_opt_string("body")?,
            pinned: row.get_bool("pinned")?,
        })
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn make_row(cols: &[&str], vals: &[serde_json::Value]) -> DbRow {
    DbRow {
        columns: cols.iter().map(|s| (*s).to_owned()).collect(),
        values: vals.to_vec(),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[test]
fn table_name_is_correct() {
    assert_eq!(Note::table_name(), "notes");
}

#[test]
fn column_names_exclude_id() {
    assert_eq!(Note::column_names(), &["title", "body", "pinned"]);
}

#[test]
fn primary_key_none_for_new_record() {
    let n = Note {
        id: None,
        title: "x".into(),
        body: None,
        pinned: false,
    };
    assert!(n.primary_key().is_none());
}

#[test]
fn set_primary_key_updates_id() {
    let mut n = Note {
        id: None,
        title: "x".into(),
        body: None,
        pinned: false,
    };
    n.set_primary_key(42);
    assert_eq!(n.primary_key(), Some(42));
}

#[test]
fn to_values_has_correct_length() {
    let n = Note {
        id: Some(1),
        title: "hello".into(),
        body: Some("world".into()),
        pinned: true,
    };
    assert_eq!(n.to_values().len(), Note::column_names().len());
}

#[test]
fn from_row_roundtrip() {
    let row = make_row(
        &["id", "title", "body", "pinned"],
        &[
            serde_json::Value::Number(7.into()),
            serde_json::Value::String("My Note".into()),
            serde_json::Value::String("Some body".into()),
            serde_json::Value::Bool(false),
        ],
    );
    let note = Note::from_row(&row).unwrap();
    assert_eq!(note.id, Some(7));
    assert_eq!(note.title, "My Note");
    assert_eq!(note.body, Some("Some body".into()));
    assert!(!note.pinned);
}

#[test]
fn from_row_null_body_maps_to_none() {
    let row = make_row(
        &["id", "title", "body", "pinned"],
        &[
            serde_json::Value::Number(1.into()),
            serde_json::Value::String("T".into()),
            serde_json::Value::Null,
            serde_json::Value::Bool(true),
        ],
    );
    let note = Note::from_row(&row).unwrap();
    assert_eq!(note.body, None);
    assert!(note.pinned);
}

#[test]
fn get_i64_error_on_missing_column() {
    let row = make_row(&["name"], &[serde_json::Value::String("x".into())]);
    assert!(row.get_i64("missing").is_err());
}

#[test]
fn get_bool_from_integer_1() {
    let row = make_row(&["flag"], &[serde_json::Value::Number(1.into())]);
    assert!(row.get_bool("flag").unwrap());
}

#[test]
fn dbrows_struct_accessible() {
    let rows = DbRows {
        rows: vec![],
        rows_affected: 0,
    };
    assert_eq!(rows.rows.len(), 0);
}
