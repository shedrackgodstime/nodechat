use anyhow::Result;
use rand::rngs::OsRng;
use rand::RngCore;
use rusqlite::Connection;
use std::path::Path;

use parking_lot::Mutex;
use std::sync::Arc;

pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Initialize the embedded SQLite database schema.
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // Ensure all required tables exist
        // Define the target schema
        let queries = [
            "CREATE TABLE IF NOT EXISTS config (
                key TEXT PRIMARY KEY,
                value BLOB
            );",
            "CREATE TABLE IF NOT EXISTS peers (
                node_id TEXT PRIMARY KEY,
                public_key_hex TEXT,
                alias TEXT,
                shared_secret BLOB,
                last_seen INTEGER
            );",
            "CREATE TABLE IF NOT EXISTS groups (
                topic_id BLOB PRIMARY KEY,
                name TEXT,
                group_key BLOB
            );",
            "CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                sender TEXT,
                recipient TEXT,
                topic_id BLOB,
                content BLOB,
                timestamp INTEGER,
                status TEXT
            );",
        ];

        for query in queries {
            conn.execute(query, [])?;
        }

        // Migration: Add public_key_hex column if it doesn't exist (for existing databases)
        let migration_result = conn.execute("ALTER TABLE peers ADD COLUMN public_key_hex TEXT", []);
        if let Err(e) = migration_result {
            if !e.to_string().contains("duplicate column name") {
                return Err(e.into());
            }
        }

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Retrieves a binary value from the persistent config table.
    pub fn get_config(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare("SELECT value FROM config WHERE key = ?")?;
        let mut rows = stmt.query([key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get::<_, Vec<u8>>(0)?))
        } else {
            Ok(None)
        }
    }

    /// Stores/Updates a binary value in the config table.
    pub fn set_config(&self, key: &str, value: &[u8]) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO config (key, value) VALUES (?, ?) 
             ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            rusqlite::params![key, value],
        )?;
        Ok(())
    }

    /// Stores or updates a peer's identity and connection info.
    pub fn upsert_peer(&self, node_id: &str, public_key: &[u8], alias: Option<&str>) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO peers (node_id, public_key_hex, alias, last_seen) 
             VALUES (?, ?, ?, strftime('%s','now'))
             ON CONFLICT(node_id) DO UPDATE SET 
                public_key_hex=excluded.public_key_hex,
                alias=COALESCE(excluded.alias, alias),
                last_seen=excluded.last_seen",
            rusqlite::params![node_id, hex::encode(public_key), alias],
        )?;
        Ok(())
    }

    /// Retrieves a peer's public key by their NodeID.
    pub fn get_peer_key(&self, node_id: &str) -> Result<Option<Vec<u8>>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare("SELECT public_key_hex FROM peers WHERE node_id = ?")?;
        let mut rows = stmt.query([node_id])?;
        if let Some(row) = rows.next()? {
            let hex_str: String = row.get::<_, String>(0)?;
            Ok(Some(hex::decode(hex_str)?))
        } else {
            Ok(None)
        }
    }

    /// Retrieves or generates the persistent identity of the Iroh node.
    pub fn get_or_create_iroh_key(&self) -> Result<iroh::SecretKey> {
        match self.get_config("iroh_secret_key")? {
            Some(bytes) => {
                let bytes_array: [u8; 32] = bytes
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("Invalid iroh_secret_key length in DB"))?;
                Ok(iroh::SecretKey::from_bytes(&bytes_array))
            }
            None => {
                let mut bytes = [0u8; 32];
                let mut rng = OsRng;
                rng.fill_bytes(&mut bytes);
                let new_key = iroh::SecretKey::from_bytes(&bytes);
                self.set_config("iroh_secret_key", &new_key.to_bytes())?;
                println!("[Storage] Generated and persisted new Iroh identity.");
                Ok(new_key)
            }
        }
    }

    /// Quick check if an identity exists in the given database file.
    pub fn has_identity<P: AsRef<Path>>(path: P) -> Result<bool> {
        let p = path.as_ref();
        if !p.exists() {
            return Ok(false);
        }
        let conn = Connection::open(p)?;
        // Use a defensive check in case the table doesn't exist yet
        let mut stmt = conn
            .prepare("SELECT count(*) FROM sqlite_master WHERE type='table' AND name='config'")?;
        let table_exists: i32 = stmt.query_row([], |r| r.get(0))?;
        if table_exists == 0 {
            return Ok(false);
        }

        let mut stmt = conn.prepare("SELECT value FROM config WHERE key = 'iroh_secret_key'")?;
        let exists = stmt.exists([])?;
        Ok(exists)
    }
}
