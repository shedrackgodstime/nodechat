//! Local SQLite database — initialisation and schema management.
//!
//! Call `initialize` once at startup. All DDL uses `CREATE TABLE IF NOT EXISTS`
//! so it is safe and idempotent on every launch.
//!
//! WAL mode and foreign key enforcement are enabled on every open.

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;

pub mod queries;

/// Open (or create) the database at `path` and apply the full schema.
///
/// WAL mode and foreign keys are enabled before any other operation.
pub fn initialize(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)
        .with_context(|| format!("failed to open database at {:?}", path))?;
    enable_pragmas(&conn)?;
    apply_schema(&conn)?;
    Ok(conn)
}

/// Open an in-memory database — used exclusively in unit tests.
#[cfg(test)]
pub fn open_in_memory() -> Result<Connection> {
    let conn = Connection::open_in_memory()
        .context("failed to open in-memory SQLite database")?;
    enable_pragmas(&conn)?;
    apply_schema(&conn)?;
    Ok(conn)
}

/// WAL journal mode + foreign key enforcement.
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
        "
        -- Singleton identity row. id is always 1.
        CREATE TABLE IF NOT EXISTS local_identity (
            id              INTEGER PRIMARY KEY CHECK (id = 1),
            display_name    TEXT    NOT NULL,
            node_id_hex     TEXT    NOT NULL,
            x25519_secret   BLOB    NOT NULL,
            endpoint_ticket TEXT    NOT NULL DEFAULT '',
            pin_hash        TEXT    NOT NULL DEFAULT ''
        );

        -- Known peers / contact book.
        -- node_id is the hex iroh NodeId — the stable primary key.
        CREATE TABLE IF NOT EXISTS peers (
            node_id         TEXT PRIMARY KEY,
            display_name    TEXT NOT NULL,
            endpoint_ticket TEXT NOT NULL DEFAULT '',
            x25519_pubkey   TEXT NOT NULL DEFAULT '',
            verified        INTEGER NOT NULL DEFAULT 0
        );

        -- Swarm groups joined locally.
        CREATE TABLE IF NOT EXISTS groups (
            topic_id        TEXT PRIMARY KEY,
            group_name      TEXT NOT NULL,
            symmetric_key   BLOB NOT NULL
        );

        -- All messages: direct and group.
        -- target_id = peers.node_id  (direct) OR groups.topic_id (group).
        -- sender_id = sender's node_id_hex. Never the string 'me'.
        -- kind: 'standard' | 'system' | 'group_invite'.
        -- status: 'queued' | 'sent' | 'delivered' | 'read' | 'failed'.
        -- invite_* columns are only populated when kind = 'group_invite'.
        CREATE TABLE IF NOT EXISTS messages (
            id                TEXT PRIMARY KEY,
            kind              TEXT NOT NULL DEFAULT 'standard',
            target_id         TEXT NOT NULL,
            sender_id         TEXT NOT NULL,
            content           TEXT NOT NULL,
            timestamp         INTEGER NOT NULL,
            status            TEXT NOT NULL DEFAULT 'queued',
            invite_topic_id   TEXT NOT NULL DEFAULT '',
            invite_group_name TEXT NOT NULL DEFAULT '',
            invite_key        TEXT NOT NULL DEFAULT ''
        );
        ",
    )
    .context("failed to apply database schema")
}
