use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        Self::enable_pragmas(&conn)?;
        Self::apply_schema(&conn)?;
        Ok(Self { conn })
    }

    pub fn memory() -> Result<Self> {
        let conn = Connection::in_memory()?;
        Self::enable_pragmas(&conn)?;
        Self::apply_schema(&conn)?;
        Ok(Self { conn })
    }

    fn enable_pragmas(conn: &Connection) -> Result<()> {
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        Ok(())
    }

    fn apply_schema(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS identity (
                id TEXT PRIMARY KEY,
                display_name TEXT NOT NULL,
                secret_key BLOB NOT NULL,
                public_key BLOB NOT NULL,
                created_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS peers (
                id TEXT PRIMARY KEY,
                display_name TEXT NOT NULL,
                public_key BLOB NOT NULL,
                ticket TEXT,
                status TEXT NOT NULL DEFAULT 'pending',
                added_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS groups (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                gossip_topic BLOB NOT NULL,
                created_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                chat_id TEXT NOT NULL,
                sender_id TEXT NOT NULL,
                ciphertext TEXT NOT NULL,
                kind TEXT NOT NULL DEFAULT 'direct',
                status TEXT NOT NULL DEFAULT 'sent',
                created_at INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_messages_chat ON messages(chat_id, created_at);
            ",
        )?;
        Ok(())
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }
}
