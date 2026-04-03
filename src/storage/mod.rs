//! SQLite database initialisation and schema management.
//!
//! Call `initialize` once at startup — all DDL uses `CREATE TABLE IF NOT EXISTS`
//! so it is safe and idempotent on every launch (RULES.md DB-05).
//!
//! WAL mode and foreign key enforcement are enabled immediately (RULES.md DB-06).

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;

pub mod queries;

/// Open (or create) the database at `path` and apply the full NodeChat schema.
///
/// WAL mode and foreign keys are enabled before any other operation.
///
/// # Errors
/// Returns an error if the file cannot be opened or schema setup fails.
pub fn initialize(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)
        .with_context(|| format!("failed to open database at {:?}", path))?;

    enable_pragmas(&conn)?;
    apply_schema(&conn)?;

    Ok(conn)
}

/// Open an in-memory database with the full schema applied.
///
/// Used exclusively in unit tests (RULES.md T-RULE-04).
#[cfg(test)]
pub fn open_in_memory() -> Result<Connection> {
    let conn = Connection::open_in_memory()
        .context("failed to open in-memory SQLite database")?;
    enable_pragmas(&conn)?;
    apply_schema(&conn)?;
    Ok(conn)
}

/// Enable WAL and foreign key enforcement (RULES.md DB-06).
fn enable_pragmas(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA foreign_keys=ON;",
    )
    .context("failed to configure database pragmas")
}

/// Create all tables if they do not already exist.
fn apply_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS local_identity (
            id                    INTEGER PRIMARY KEY CHECK (id = 1),
            display_name          TEXT NOT NULL,
            node_id_bytes         BLOB NOT NULL,
            x25519_secret         BLOB NOT NULL
        );

        CREATE TABLE IF NOT EXISTS peers (
            node_id       TEXT PRIMARY KEY,
            display_name  TEXT NOT NULL,
            endpoint_ticket TEXT NOT NULL DEFAULT '',
            x25519_pubkey TEXT NOT NULL,
            verified      INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS groups (
            topic_id      TEXT PRIMARY KEY,
            group_name    TEXT NOT NULL,
            -- symmetric_key is encrypted at rest under the local password key (RULES.md C-05)
            symmetric_key BLOB NOT NULL
        );

        CREATE TABLE IF NOT EXISTS messages (
            id        TEXT PRIMARY KEY,
            -- 'direct' | 'group' | 'file' | 'group_invite'
            type      TEXT NOT NULL,
            -- NodeId for 1:1 messages, TopicId for group messages
            target_id TEXT NOT NULL,
            sender_id TEXT NOT NULL,
            -- Decrypted plaintext stored locally (RULES.md C-06)
            content   TEXT NOT NULL,
            -- UTC Unix seconds (RULES.md U-07)
            timestamp INTEGER NOT NULL,
            -- 'queued' | 'sent' | 'delivered' | 'read'
            status    TEXT NOT NULL
        );",
    )
    .context("failed to apply database schema")
    .and_then(|_| ensure_peer_schema(conn))
}

fn ensure_peer_schema(conn: &Connection) -> Result<()> {
    let mut stmt = conn
        .prepare("PRAGMA table_info(peers)")
        .context("failed to inspect peers schema")?;
    let mut rows = stmt.query([])?;
    let mut has_endpoint_ticket = false;

    while let Some(row) = rows.next().context("failed to read peers schema row")? {
        let name: String = row.get(1).context("schema column name")?;
        if name == "endpoint_ticket" {
            has_endpoint_ticket = true;
            break;
        }
    }

    if !has_endpoint_ticket {
        conn.execute_batch(
            "ALTER TABLE peers ADD COLUMN endpoint_ticket TEXT NOT NULL DEFAULT '';",
        )
        .context("failed to add endpoint_ticket column to peers")?;
    }

    Ok(())
}
