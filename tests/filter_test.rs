// Unit tests for Filter<T> SQL generation and combinators.

use fs_db::filter::{Filter, Order};

// Phantom type for tests — no real table needed.
struct User;

#[test]
fn eq_produces_where_clause() {
    let (sql, params) = Filter::<User>::eq("name", "alice").to_sql();
    assert_eq!(sql, "WHERE name = ?");
    assert_eq!(params.len(), 1);
    assert_eq!(params[0], "alice");
}

#[test]
fn ne_produces_where_clause() {
    let (sql, params) = Filter::<User>::ne("status", "deleted").to_sql();
    assert_eq!(sql, "WHERE status != ?");
    assert_eq!(params.len(), 1);
}

#[test]
fn gt_produces_where_clause() {
    let (sql, _params) = Filter::<User>::gt("age", 18_i64).to_sql();
    assert_eq!(sql, "WHERE age > ?");
}

#[test]
fn lt_produces_where_clause() {
    let (sql, _params) = Filter::<User>::lt("score", 100_i64).to_sql();
    assert_eq!(sql, "WHERE score < ?");
}

#[test]
fn gte_produces_where_clause() {
    let (sql, _params) = Filter::<User>::gte("priority", 5_i64).to_sql();
    assert_eq!(sql, "WHERE priority >= ?");
}

#[test]
fn lte_produces_where_clause() {
    let (sql, _params) = Filter::<User>::lte("priority", 5_i64).to_sql();
    assert_eq!(sql, "WHERE priority <= ?");
}

#[test]
fn in_list_produces_placeholders() {
    let (sql, params) = Filter::<User>::in_list("id", vec![1_i64, 2_i64, 3_i64]).to_sql();
    assert_eq!(sql, "WHERE id IN (?, ?, ?)");
    assert_eq!(params.len(), 3);
}

#[test]
fn like_produces_where_clause() {
    let (sql, params) = Filter::<User>::like("email", "%@example.com").to_sql();
    assert_eq!(sql, "WHERE email LIKE ?");
    assert_eq!(params[0], "%@example.com");
}

#[test]
fn is_null_produces_where_clause() {
    let (sql, params) = Filter::<User>::is_null("deleted_at").to_sql();
    assert_eq!(sql, "WHERE deleted_at IS NULL");
    assert!(params.is_empty());
}

#[test]
fn and_combines_conditions() {
    let f = Filter::<User>::eq("status", "active").and(Filter::ne("deleted", true));
    let (sql, params) = f.to_sql();
    assert_eq!(sql, "WHERE status = ? AND deleted != ?");
    assert_eq!(params.len(), 2);
}

#[test]
fn order_by_ascending_appended() {
    let f = Filter::<User>::eq("status", "active").order_by("name", Order::Ascending);
    let (sql, _params) = f.to_sql();
    assert!(sql.contains("ORDER BY name ASC"), "sql={sql}");
}

#[test]
fn order_by_descending_appended() {
    let f = Filter::<User>::eq("status", "active").order_by("created_at", Order::Descending);
    let (sql, _params) = f.to_sql();
    assert!(sql.contains("ORDER BY created_at DESC"), "sql={sql}");
}

#[test]
fn limit_appended() {
    let f = Filter::<User>::eq("x", "y").limit(10);
    let (sql, _) = f.to_sql();
    assert!(sql.contains("LIMIT 10"), "sql={sql}");
}

#[test]
fn offset_appended_when_nonzero() {
    let f = Filter::<User>::eq("x", "y").offset(20);
    let (sql, _) = f.to_sql();
    assert!(sql.contains("OFFSET 20"), "sql={sql}");
}

#[test]
fn empty_filter_produces_no_where() {
    let f: Filter<User> = Filter::default();
    let (sql, params) = f.to_sql();
    assert!(sql.is_empty(), "sql should be empty, got: {sql}");
    assert!(params.is_empty());
}

#[test]
fn full_query_correct_order() {
    let f = Filter::<User>::eq("active", true)
        .order_by("name", Order::Ascending)
        .limit(5)
        .offset(10);
    let (sql, _) = f.to_sql();
    let where_pos = sql.find("WHERE").unwrap_or(usize::MAX);
    let order_pos = sql.find("ORDER BY").unwrap_or(usize::MAX);
    let limit_pos = sql.find("LIMIT").unwrap_or(usize::MAX);
    let offset_pos = sql.find("OFFSET").unwrap_or(usize::MAX);
    assert!(where_pos < order_pos, "WHERE must come before ORDER BY");
    assert!(order_pos < limit_pos, "ORDER BY must come before LIMIT");
    assert!(limit_pos < offset_pos, "LIMIT must come before OFFSET");
}
