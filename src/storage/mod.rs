use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;

pub struct Database {
    conn: Connection,
}

impl Database {
    /// Initialize the embedded SQLite database schema.
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        
        // Ensure all required tables exist
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS peers (
                node_id TEXT PRIMARY KEY,
                display_name TEXT NOT NULL,
                x25519_pubkey TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS groups (
                topic_id TEXT PRIMARY KEY,
                group_name TEXT NOT NULL,
                symmetric_key BLOB NOT NULL
            );

            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                type TEXT NOT NULL,
                target_id TEXT NOT NULL,
                sender_id TEXT NOT NULL, 
                content BLOB NOT NULL,
                timestamp INTEGER NOT NULL,
                status TEXT NOT NULL
            );
            "
        )?;

        Ok(Self { conn })
    }
}
