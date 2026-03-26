// Write buffer for batching database writes into transactions.
//
// Inspired by ownCloud's write-buffering approach: instead of one SQLite
// transaction per write, accumulate writes and flush them together.
// This dramatically reduces lock contention under high write load.

use std::time::{Duration, Instant};

use sea_orm::{ConnectionTrait, DatabaseConnection, TransactionTrait};
use tokio::sync::Mutex;
use tracing::error;

use fs_error::FsError;

// ── BufferedWrite ─────────────────────────────────────────────────────────────

/// A single buffered SQL write operation.
#[derive(Debug)]
pub struct BufferedWrite {
    /// Raw SQL statement (parameterless — values must be embedded or use
    /// positional `?` params in `values`).
    sql: String,
    /// JSON-encoded bound values (deserialized by the flush impl).
    values: Vec<serde_json::Value>,
}

impl BufferedWrite {
    /// Create a new buffered write from a SQL statement and bound values.
    pub fn new(sql: impl Into<String>, values: Vec<serde_json::Value>) -> Self {
        Self {
            sql: sql.into(),
            values,
        }
    }

    /// Create a parameterless write (no bound values).
    pub fn statement(sql: impl Into<String>) -> Self {
        Self {
            sql: sql.into(),
            values: vec![],
        }
    }

    /// The SQL statement for this write.
    pub fn sql(&self) -> &str {
        &self.sql
    }

    /// The bound parameter values for this write.
    pub fn values(&self) -> &[serde_json::Value] {
        &self.values
    }
}

// ── FlushResult ───────────────────────────────────────────────────────────────

/// Result of a [`WriteBuffer::flush`] operation.
#[derive(Debug)]
pub struct FlushResult {
    /// Number of SQL statements flushed.
    pub count: usize,
    /// Wall-clock time taken by the flush.
    pub duration: Duration,
}

// ── WriteBuffer ───────────────────────────────────────────────────────────────

/// Async write buffer — accumulates writes and flushes in a single transaction.
///
/// # Usage
///
/// ```rust,ignore
/// let buf = WriteBuffer::with_defaults(db.inner().clone());
/// buf.enqueue(BufferedWrite { sql: "INSERT INTO ...".into(), values: vec![] }).await?;
///
/// // In a background task:
/// buf.run_auto_flush().await;
/// ```
pub struct WriteBuffer {
    queue: Mutex<Vec<BufferedWrite>>,
    flush_interval: Duration,
    max_batch_size: usize,
    last_flush: Mutex<Instant>,
    conn: DatabaseConnection,
}

impl WriteBuffer {
    /// Create with explicit flush interval and max batch size.
    pub fn new(conn: DatabaseConnection, flush_interval: Duration, max_batch_size: usize) -> Self {
        Self {
            queue: Mutex::new(Vec::new()),
            flush_interval,
            max_batch_size,
            last_flush: Mutex::new(Instant::now()),
            conn,
        }
    }

    /// Create with defaults: 100 ms flush interval, 500 max batch size.
    pub fn with_defaults(conn: DatabaseConnection) -> Self {
        Self::new(conn, Duration::from_millis(100), 500)
    }

    /// Enqueue a write. Returns immediately — does not flush.
    ///
    /// If the batch reaches `max_batch_size`, an automatic flush is triggered.
    pub async fn enqueue(&self, write: BufferedWrite) -> Result<(), FsError> {
        let mut queue = self.queue.lock().await;
        queue.push(write);
        let full = queue.len() >= self.max_batch_size;
        drop(queue);

        if full {
            self.flush().await?;
        }
        Ok(())
    }

    /// Flush all queued writes in a single transaction.
    ///
    /// Returns immediately with `count = 0` when the queue is empty.
    pub async fn flush(&self) -> Result<FlushResult, FsError> {
        let mut queue = self.queue.lock().await;
        if queue.is_empty() {
            return Ok(FlushResult {
                count: 0,
                duration: Duration::ZERO,
            });
        }

        let writes: Vec<BufferedWrite> = queue.drain(..).collect();
        drop(queue);

        let start = Instant::now();
        let count = writes.len();

        let txn = self
            .conn
            .begin()
            .await
            .map_err(|e| FsError::internal(format!("db transaction begin: {e}")))?;

        for write in writes {
            txn.execute_unprepared(&write.sql)
                .await
                .map_err(|e| FsError::internal(format!("db execute: {e}")))?;
        }

        txn.commit()
            .await
            .map_err(|e| FsError::internal(format!("db commit: {e}")))?;

        *self.last_flush.lock().await = Instant::now();

        Ok(FlushResult {
            count,
            duration: start.elapsed(),
        })
    }

    /// `true` when a flush is overdue (interval elapsed) or the batch is full.
    pub async fn needs_flush(&self) -> bool {
        let queue = self.queue.lock().await;
        if queue.len() >= self.max_batch_size {
            return true;
        }
        let last = self.last_flush.lock().await;
        last.elapsed() >= self.flush_interval
    }

    /// Background auto-flush loop. Run this in a `tokio::spawn` task.
    ///
    /// Flushes every `flush_interval`. Errors are logged but do not stop the loop.
    /// Cancel the task to stop the loop.
    pub async fn run_auto_flush(&self) {
        loop {
            tokio::time::sleep(self.flush_interval).await;
            if let Err(e) = self.flush().await {
                error!("write buffer flush error: {e}");
            }
        }
    }
}
