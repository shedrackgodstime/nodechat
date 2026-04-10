//! All SQL CRUD for NodeChat-New.
//!
//! Rules:
//! - Every SQL string lives in this file only.
//! - Every function returns `Result<T>`.
//! - Status transitions are forward-only (queued → sent → delivered → read).
//! - `INSERT OR IGNORE` for messages (idempotent receive).
//! - `ON CONFLICT DO UPDATE` for peers (upsert on reconnect).

use anyhow::{bail, Context, Result};
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::contract::MessageStatus;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Parse a status string from the DB into a `MessageStatus`.
fn parse_status(s: &str) -> Result<MessageStatus> {
    match s {
        "queued"    => Ok(MessageStatus::Queued),
        "sent"      => Ok(MessageStatus::Sent),
        "delivered" => Ok(MessageStatus::Delivered),
        "read"      => Ok(MessageStatus::Read),
        other       => bail!("unknown message status in DB: {:?}", other),
    }
}

/// Derive a two-letter avatar label from a display name.
pub fn derive_initials(name: &str) -> String {
    let mut out = String::new();
    for word in name.split_whitespace() {
        if let Some(ch) = word.chars().next() {
            out.push(ch.to_ascii_uppercase());
        }
        if out.chars().count() >= 2 {
            break;
        }
    }
    if out.is_empty() {
        name.chars().take(2).collect::<String>().to_ascii_uppercase()
    } else {
        out
    }
}

// ── Record Types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct LocalIdentityRecord {
    pub display_name:    String,
    pub node_id_hex:     String,
    pub x25519_secret:   Vec<u8>,
    pub endpoint_ticket: String,
    pub pin_hash:        String,
}

#[derive(Debug, Clone)]
pub struct PeerRecord {
    /// Hex-encoded iroh NodeId — the stable primary key.
    pub node_id:         String,
    pub display_name:    String,
    pub endpoint_ticket: String,
    pub x25519_pubkey:   String,
    pub verified:        bool,
}

#[derive(Debug, Clone)]
pub struct GroupRecord {
    /// Hex-encoded iroh TopicId — the primary key.
    pub topic_id:      String,
    pub group_name:    String,
    pub symmetric_key: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct MessageRecord {
    pub id:                Uuid,
    pub kind:              String,   // "standard" | "system" | "group_invite"
    pub target_id:         String,
    pub sender_id:         String,   // actual node_id_hex, never the literal "me"
    pub content:           String,
    pub timestamp:         i64,      // UTC Unix seconds
    pub status:            MessageStatus,
    pub invite_topic_id:   String,
    pub invite_group_name: String,
    pub invite_key:        String,
}

/// A flat row used to build the home-screen chat list.
#[derive(Debug, Clone)]
pub struct ChatPreviewRecord {
    pub id:                  String,
    pub title:               String,
    pub initials:            String,
    pub is_group:            bool,
    pub last_message:        String,
    pub last_message_status: MessageStatus,
    pub is_outgoing:         bool,
    pub timestamp:           i64,     // raw, caller formats for display
    pub is_verified:         bool,
    pub has_queued:          bool,
}

// ── Local Identity ────────────────────────────────────────────────────────────

/// Insert the local identity. Only one row can ever exist (id = 1).
pub fn insert_local_identity(conn: &Connection, r: &LocalIdentityRecord) -> Result<()> {
    conn.execute(
        "INSERT INTO local_identity
             (id, display_name, node_id_hex, x25519_secret, endpoint_ticket, pin_hash)
         VALUES (1, ?1, ?2, ?3, ?4, ?5)",
        params![r.display_name, r.node_id_hex, r.x25519_secret, r.endpoint_ticket, r.pin_hash],
    )
    .context("insert_local_identity failed")?;
    Ok(())
}

/// Fetch the local identity row if it exists.
pub fn get_local_identity(conn: &Connection) -> Result<Option<LocalIdentityRecord>> {
    let mut stmt = conn
        .prepare(
            "SELECT display_name, node_id_hex, x25519_secret, endpoint_ticket, pin_hash
             FROM local_identity WHERE id = 1",
        )
        .context("prepare get_local_identity")?;

    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next().context("read local_identity row")? {
        Ok(Some(LocalIdentityRecord {
            display_name:    row.get(0).context("display_name")?,
            node_id_hex:     row.get(1).context("node_id_hex")?,
            x25519_secret:   row.get(2).context("x25519_secret")?,
            endpoint_ticket: row.get(3).context("endpoint_ticket")?,
            pin_hash:        row.get(4).context("pin_hash")?,
        }))
    } else {
        Ok(None)
    }
}

pub fn update_display_name(conn: &Connection, name: &str) -> Result<()> {
    conn.execute(
        "UPDATE local_identity SET display_name = ?1 WHERE id = 1",
        params![name],
    )
    .context("update_display_name failed")?;
    Ok(())
}

pub fn update_endpoint_ticket(conn: &Connection, ticket: &str) -> Result<()> {
    conn.execute(
        "UPDATE local_identity SET endpoint_ticket = ?1 WHERE id = 1",
        params![ticket],
    )
    .context("update_endpoint_ticket failed")?;
    Ok(())
}

pub fn update_pin_hash(conn: &Connection, hash: &str) -> Result<()> {
    conn.execute(
        "UPDATE local_identity SET pin_hash = ?1 WHERE id = 1",
        params![hash],
    )
    .context("update_pin_hash failed")?;
    Ok(())
}

/// Wipe all user data. Used by ResetIdentity.
pub fn delete_all(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "DELETE FROM messages;
         DELETE FROM groups;
         DELETE FROM peers;
         DELETE FROM local_identity;",
    )
    .context("delete_all failed")
}

// ── Peers ─────────────────────────────────────────────────────────────────────

/// Insert a new peer, or update display_name + ticket on conflict.
pub fn insert_peer(conn: &Connection, r: &PeerRecord) -> Result<()> {
    conn.execute(
        "INSERT INTO peers (node_id, display_name, endpoint_ticket, x25519_pubkey, verified)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(node_id) DO UPDATE SET
             display_name    = excluded.display_name,
             endpoint_ticket = excluded.endpoint_ticket",
        params![r.node_id, r.display_name, r.endpoint_ticket, r.x25519_pubkey, r.verified as i32],
    )
    .context("insert_peer failed")?;
    Ok(())
}

pub fn get_peer(conn: &Connection, node_id: &str) -> Result<Option<PeerRecord>> {
    let mut stmt = conn
        .prepare(
            "SELECT node_id, display_name, endpoint_ticket, x25519_pubkey, verified
             FROM peers WHERE node_id = ?1",
        )
        .context("prepare get_peer")?;
    let mut rows = stmt.query(params![node_id])?;
    if let Some(row) = rows.next().context("read peer row")? {
        Ok(Some(PeerRecord {
            node_id:         row.get(0)?,
            display_name:    row.get(1)?,
            endpoint_ticket: row.get(2)?,
            x25519_pubkey:   row.get(3)?,
            verified:        row.get::<_, i32>(4)? != 0,
        }))
    } else {
        Ok(None)
    }
}

pub fn list_peers(conn: &Connection) -> Result<Vec<PeerRecord>> {
    let mut stmt = conn
        .prepare(
            "SELECT node_id, display_name, endpoint_ticket, x25519_pubkey, verified
             FROM peers ORDER BY display_name ASC",
        )
        .context("prepare list_peers")?;
    let rows = stmt
        .query_map([], |row| {
            Ok(PeerRecord {
                node_id:         row.get(0)?,
                display_name:    row.get(1)?,
                endpoint_ticket: row.get(2)?,
                x25519_pubkey:   row.get(3)?,
                verified:        row.get::<_, i32>(4)? != 0,
            })
        })
        .context("query list_peers")?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("collect list_peers")?;
    Ok(rows)
}

pub fn update_peer_ticket(conn: &Connection, node_id: &str, ticket: &str) -> Result<()> {
    let n = conn
        .execute(
            "UPDATE peers SET endpoint_ticket = ?2 WHERE node_id = ?1",
            params![node_id, ticket],
        )
        .context("update_peer_ticket failed")?;
    if n == 0 { bail!("update_peer_ticket: peer {:?} not found", node_id); }
    Ok(())
}

pub fn update_peer_pubkey(conn: &Connection, node_id: &str, pubkey: &str) -> Result<()> {
    let n = conn
        .execute(
            "UPDATE peers SET x25519_pubkey = ?2 WHERE node_id = ?1",
            params![node_id, pubkey],
        )
        .context("update_peer_pubkey failed")?;
    if n == 0 { bail!("update_peer_pubkey: peer {:?} not found", node_id); }
    Ok(())
}

pub fn set_peer_verified(conn: &Connection, node_id: &str, verified: bool) -> Result<()> {
    let n = conn
        .execute(
            "UPDATE peers SET verified = ?2 WHERE node_id = ?1",
            params![node_id, verified as i32],
        )
        .context("set_peer_verified failed")?;
    if n == 0 { bail!("set_peer_verified: peer {:?} not found", node_id); }
    Ok(())
}

pub fn delete_peer(conn: &Connection, node_id: &str) -> Result<()> {
    conn.execute("DELETE FROM peers WHERE node_id = ?1", params![node_id])
        .context("delete_peer failed")?;
    Ok(())
}

// ── Groups ────────────────────────────────────────────────────────────────────

pub fn insert_group(conn: &Connection, r: &GroupRecord) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO groups (topic_id, group_name, symmetric_key)
         VALUES (?1, ?2, ?3)",
        params![r.topic_id, r.group_name, r.symmetric_key],
    )
    .context("insert_group failed")?;
    Ok(())
}

pub fn get_group(conn: &Connection, topic_id: &str) -> Result<Option<GroupRecord>> {
    let mut stmt = conn
        .prepare("SELECT topic_id, group_name, symmetric_key FROM groups WHERE topic_id = ?1")
        .context("prepare get_group")?;
    let mut rows = stmt.query(params![topic_id])?;
    if let Some(row) = rows.next().context("read group row")? {
        Ok(Some(GroupRecord {
            topic_id:      row.get(0)?,
            group_name:    row.get(1)?,
            symmetric_key: row.get(2)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn list_groups(conn: &Connection) -> Result<Vec<GroupRecord>> {
    let mut stmt = conn
        .prepare("SELECT topic_id, group_name, symmetric_key FROM groups ORDER BY group_name ASC")
        .context("prepare list_groups")?;
    let rows = stmt
        .query_map([], |row| {
            Ok(GroupRecord {
                topic_id:      row.get(0)?,
                group_name:    row.get(1)?,
                symmetric_key: row.get(2)?,
            })
        })
        .context("query list_groups")?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("collect list_groups")?;
    Ok(rows)
}

pub fn delete_group(conn: &Connection, topic_id: &str) -> Result<()> {
    conn.execute("DELETE FROM groups WHERE topic_id = ?1", params![topic_id])
        .context("delete_group failed")?;
    Ok(())
}

/// Check whether a topic_id is already in the local groups table.
/// Used to derive `invite_is_joined`.
pub fn group_exists(conn: &Connection, topic_id: &str) -> Result<bool> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM groups WHERE topic_id = ?1",
            params![topic_id],
            |row| row.get(0),
        )
        .context("group_exists query failed")?;
    Ok(count > 0)
}

// ── Messages ──────────────────────────────────────────────────────────────────

pub fn insert_message(conn: &Connection, r: &MessageRecord) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO messages
             (id, kind, target_id, sender_id, content, timestamp, status,
              invite_topic_id, invite_group_name, invite_key)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            r.id.to_string(),
            r.kind,
            r.target_id,
            r.sender_id,
            r.content,
            r.timestamp,
            r.status.as_str(),
            r.invite_topic_id,
            r.invite_group_name,
            r.invite_key,
        ],
    )
    .context("insert_message failed")?;
    Ok(())
}

/// Fetch all messages for a conversation, oldest first.
pub fn list_messages(conn: &Connection, target_id: &str) -> Result<Vec<MessageRecord>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, kind, target_id, sender_id, content, timestamp, status,
                    invite_topic_id, invite_group_name, invite_key
             FROM messages WHERE target_id = ?1 ORDER BY timestamp ASC",
        )
        .context("prepare list_messages")?;
    collect_messages(stmt.query_map(params![target_id], map_message_row)?)
}

/// Fetch only queued / sent messages for a given conversation (for retry).
pub fn list_queued(conn: &Connection, target_id: &str) -> Result<Vec<MessageRecord>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, kind, target_id, sender_id, content, timestamp, status,
                    invite_topic_id, invite_group_name, invite_key
             FROM messages
             WHERE target_id = ?1 AND status IN ('queued', 'sent')
             ORDER BY timestamp ASC",
        )
        .context("prepare list_queued")?;
    collect_messages(stmt.query_map(params![target_id], map_message_row)?)
}

/// Returns true if any queued message exists for `target_id`.
pub fn has_queued(conn: &Connection, target_id: &str) -> Result<bool> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM messages WHERE target_id = ?1 AND status = 'queued'",
            params![target_id],
            |row| row.get(0),
        )
        .context("has_queued query failed")?;
    Ok(count > 0)
}

/// Advance the delivery status of a message — forward-only transitions enforced.
pub fn advance_status(conn: &Connection, id: &Uuid, new_status: MessageStatus) -> Result<()> {
    let current_str: String = conn
        .query_row(
            "SELECT status FROM messages WHERE id = ?1",
            params![id.to_string()],
            |row| row.get(0),
        )
        .with_context(|| format!("advance_status: message {} not found", id))?;

    let current = parse_status(&current_str)?;

    // Forward-only guard
    let ok = matches!(
        (current, new_status),
        (MessageStatus::Queued,    MessageStatus::Sent)
        | (MessageStatus::Sent,      MessageStatus::Delivered)
        | (MessageStatus::Delivered, MessageStatus::Read)
        // Allow queued → delivered for cases where we get an ACK before a sent receipt
        | (MessageStatus::Queued,    MessageStatus::Delivered)
    );
    if !ok {
        bail!(
            "invalid status transition {:?} → {:?} for message {}",
            current_str, new_status.as_str(), id
        );
    }

    conn.execute(
        "UPDATE messages SET status = ?1 WHERE id = ?2",
        params![new_status.as_str(), id.to_string()],
    )
    .context("advance_status update failed")?;
    Ok(())
}

/// Delete all messages across all conversations.
pub fn clear_messages(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM messages", []).context("clear_messages failed")?;
    Ok(())
}

/// Delete all messages in one conversation without removing the peer/group.
pub fn clear_conversation(conn: &Connection, target_id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM messages WHERE target_id = ?1",
        params![target_id],
    )
    .context("clear_conversation failed")?;
    Ok(())
}

/// Delete a conversation and its peer or group row entirely.
pub fn delete_conversation(conn: &Connection, target_id: &str, is_group: bool) -> Result<()> {
    clear_conversation(conn, target_id)?;
    if is_group {
        delete_group(conn, target_id)?;
    } else {
        delete_peer(conn, target_id)?;
    }
    Ok(())
}

// ── Chat List (composite) ─────────────────────────────────────────────────────

/// Build the home-screen chat previews from peers, groups, and message history.
/// `local_node_id` is used to derive `is_outgoing` on the last message.
pub fn list_chat_previews(
    conn: &Connection,
    local_node_id: &str,
) -> Result<Vec<ChatPreviewRecord>> {
    let mut previews = Vec::new();

    // Direct peer chats
    for peer in list_peers(conn)? {
        let msgs = list_messages(conn, &peer.node_id)?;
        let last = msgs.last();
        previews.push(ChatPreviewRecord {
            id:                  peer.node_id.clone(),
            title:               peer.display_name.clone(),
            initials:            derive_initials(&peer.display_name),
            is_group:            false,
            last_message:        last.map(|m| m.content.clone()).unwrap_or_default(),
            last_message_status: last.map(|m| m.status).unwrap_or(MessageStatus::Sent),
            is_outgoing:         last.map(|m| m.sender_id == local_node_id).unwrap_or(false),
            timestamp:           last.map(|m| m.timestamp).unwrap_or(0),
            is_verified:         peer.verified,
            has_queued:          has_queued(conn, &peer.node_id)?,
        });
    }

    // Group chats
    for group in list_groups(conn)? {
        let msgs = list_messages(conn, &group.topic_id)?;
        let last = msgs.last();
        previews.push(ChatPreviewRecord {
            id:                  group.topic_id.clone(),
            title:               group.group_name.clone(),
            initials:            derive_initials(&group.group_name),
            is_group:            true,
            last_message:        last.map(|m| m.content.clone()).unwrap_or_default(),
            last_message_status: last.map(|m| m.status).unwrap_or(MessageStatus::Sent),
            is_outgoing:         last.map(|m| m.sender_id == local_node_id).unwrap_or(false),
            timestamp:           last.map(|m| m.timestamp).unwrap_or(0),
            is_verified:         true,   // groups are always trusted locally
            has_queued:          false,  // groups don't queue
        });
    }

    // Sort by most recent message first
    previews.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(previews)
}

// ── Internal helpers ──────────────────────────────────────────────────────────

type MsgRow = (String, String, String, String, String, i64, String, String, String, String);

fn map_message_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<MsgRow> {
    Ok((
        row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?,
        row.get(5)?, row.get(6)?, row.get(7)?, row.get(8)?, row.get(9)?,
    ))
}

fn collect_messages(
    mapped: rusqlite::MappedRows<'_, impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<MsgRow>>,
) -> Result<Vec<MessageRecord>> {
    mapped
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("collect messages")?
        .into_iter()
        .map(|(id, kind, target_id, sender_id, content, timestamp, status_str,
               invite_topic_id, invite_group_name, invite_key)| {
            Ok(MessageRecord {
                id: Uuid::parse_str(&id).with_context(|| format!("invalid UUID {:?}", id))?,
                kind,
                target_id,
                sender_id,
                content,
                timestamp,
                status: parse_status(&status_str)?,
                invite_topic_id,
                invite_group_name,
                invite_key,
            })
        })
        .collect()
}

// ── Unit Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::open_in_memory;

    fn identity() -> LocalIdentityRecord {
        LocalIdentityRecord {
            display_name:    "Alice".to_string(),
            node_id_hex:     "aabbcc001122".to_string(),
            x25519_secret:   vec![0u8; 32],
            endpoint_ticket: "ticket-abc".to_string(),
            pin_hash:        "".to_string(),
        }
    }

    fn peer() -> PeerRecord {
        PeerRecord {
            node_id:         "ddeeff334455".to_string(),
            display_name:    "Bob".to_string(),
            endpoint_ticket: "ticket-bob".to_string(),
            x25519_pubkey:   "bbccdd".to_string(),
            verified:        false,
        }
    }

    fn message(target: &str, sender: &str, status: MessageStatus) -> MessageRecord {
        MessageRecord {
            id:                Uuid::new_v4(),
            kind:              "standard".to_string(),
            target_id:         target.to_string(),
            sender_id:         sender.to_string(),
            content:           "hello".to_string(),
            timestamp:         1_700_000_000,
            status,
            invite_topic_id:   String::new(),
            invite_group_name: String::new(),
            invite_key:        String::new(),
        }
    }

    #[test]
    fn identity_roundtrip() {
        let conn = open_in_memory().unwrap();
        let rec = identity();
        insert_local_identity(&conn, &rec).unwrap();
        let got = get_local_identity(&conn).unwrap().expect("should exist");
        assert_eq!(got.display_name, "Alice");
        assert_eq!(got.node_id_hex,  "aabbcc001122");
        assert_eq!(got.endpoint_ticket, "ticket-abc");
    }

    #[test]
    fn peer_upsert_updates_name() {
        let conn = open_in_memory().unwrap();
        insert_peer(&conn, &peer()).unwrap();
        let mut updated = peer();
        updated.display_name = "Bobby".to_string();
        insert_peer(&conn, &updated).unwrap(); // upsert
        let got = get_peer(&conn, "ddeeff334455").unwrap().expect("should exist");
        assert_eq!(got.display_name, "Bobby");
    }

    #[test]
    fn verified_toggle() {
        let conn = open_in_memory().unwrap();
        insert_peer(&conn, &peer()).unwrap();
        set_peer_verified(&conn, "ddeeff334455", true).unwrap();
        assert!(get_peer(&conn, "ddeeff334455").unwrap().unwrap().verified);
    }

    #[test]
    fn message_insert_and_list() {
        let conn = open_in_memory().unwrap();
        insert_peer(&conn, &peer()).unwrap();
        let msg = message("ddeeff334455", "aabbcc001122", MessageStatus::Queued);
        insert_message(&conn, &msg).unwrap();
        let msgs = list_messages(&conn, "ddeeff334455").unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].status, MessageStatus::Queued);
    }

    #[test]
    fn message_idempotent_insert() {
        let conn = open_in_memory().unwrap();
        insert_peer(&conn, &peer()).unwrap();
        let msg = message("ddeeff334455", "aabbcc001122", MessageStatus::Sent);
        insert_message(&conn, &msg).unwrap();
        insert_message(&conn, &msg).unwrap(); // second insert must be ignored
        assert_eq!(list_messages(&conn, "ddeeff334455").unwrap().len(), 1);
    }

    #[test]
    fn status_advances_forward() {
        let conn = open_in_memory().unwrap();
        insert_peer(&conn, &peer()).unwrap();
        let msg = message("ddeeff334455", "aabbcc001122", MessageStatus::Queued);
        insert_message(&conn, &msg).unwrap();
        advance_status(&conn, &msg.id, MessageStatus::Sent).unwrap();
        advance_status(&conn, &msg.id, MessageStatus::Delivered).unwrap();
        let msgs = list_messages(&conn, "ddeeff334455").unwrap();
        assert_eq!(msgs[0].status, MessageStatus::Delivered);
    }

    #[test]
    fn status_backward_transition_rejected() {
        let conn = open_in_memory().unwrap();
        insert_peer(&conn, &peer()).unwrap();
        let msg = message("ddeeff334455", "aabbcc001122", MessageStatus::Delivered);
        insert_message(&conn, &msg).unwrap();
        assert!(advance_status(&conn, &msg.id, MessageStatus::Queued).is_err());
    }

    #[test]
    fn has_queued_reflects_status() {
        let conn = open_in_memory().unwrap();
        insert_peer(&conn, &peer()).unwrap();
        let msg = message("ddeeff334455", "aabbcc001122", MessageStatus::Queued);
        insert_message(&conn, &msg).unwrap();
        assert!(has_queued(&conn, "ddeeff334455").unwrap());
        advance_status(&conn, &msg.id, MessageStatus::Sent).unwrap();
        // 'sent' is not 'queued' so has_queued returns false
        assert!(!has_queued(&conn, "ddeeff334455").unwrap());
    }

    #[test]
    fn delete_conversation_removes_peer_and_messages() {
        let conn = open_in_memory().unwrap();
        insert_peer(&conn, &peer()).unwrap();
        let msg = message("ddeeff334455", "aabbcc001122", MessageStatus::Sent);
        insert_message(&conn, &msg).unwrap();
        delete_conversation(&conn, "ddeeff334455", false).unwrap();
        assert!(get_peer(&conn, "ddeeff334455").unwrap().is_none());
        assert!(list_messages(&conn, "ddeeff334455").unwrap().is_empty());
    }

    #[test]
    fn chat_preview_sorted_by_timestamp() {
        let conn = open_in_memory().unwrap();
        let p1 = peer();
        let mut p2 = peer();
        p2.node_id = "ffffff000000".to_string();
        p2.display_name = "Carol".to_string();
        insert_peer(&conn, &p1).unwrap();
        insert_peer(&conn, &p2).unwrap();

        let mut m1 = message("ddeeff334455", "aabbcc001122", MessageStatus::Sent);
        m1.timestamp = 1_000;
        let mut m2 = message("ffffff000000", "aabbcc001122", MessageStatus::Sent);
        m2.timestamp = 2_000;
        insert_message(&conn, &m1).unwrap();
        insert_message(&conn, &m2).unwrap();

        let previews = list_chat_previews(&conn, "aabbcc001122").unwrap();
        // Most recent first
        assert_eq!(previews[0].id, "ffffff000000");
        assert_eq!(previews[1].id, "ddeeff334455");
    }
}
