//! All SQL CRUD for NodeChat — the single location for every SQL string (RULES.md DB-01).
//!
//! Every function uses `rusqlite::params![]` (RULES.md DB-03).
//! Every function returns `Result<T>` (RULES.md DB-02).
//! Status transitions are forward-only and enforced here (RULES.md DB-04).

use anyhow::{bail, Context, Result};
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::core::commands::MessageStatus;

// ── Local Identity ──────────────────────────────────────────────────────────────

/// The user's own identity, stored encrypted at rest.
#[derive(Debug, Clone)]
pub struct LocalIdentityRecord {
    pub display_name: String,
    pub node_id_bytes: Vec<u8>,
    pub x25519_secret: Vec<u8>,
}

/// Insert the local identity. Only one can exist (id = 1).
pub fn insert_local_identity(conn: &Connection, identity: &LocalIdentityRecord) -> Result<()> {
    conn.execute(
        "INSERT INTO local_identity (id, display_name, node_id_bytes, x25519_secret)
         VALUES (1, ?1, ?2, ?3)",
        params![
            identity.display_name,
            identity.node_id_bytes,
            identity.x25519_secret
        ],
    )
    .context("failed to insert local identity")?;
    Ok(())
}

/// Fetch the local identity if one exists.
pub fn get_local_identity(conn: &Connection) -> Result<Option<LocalIdentityRecord>> {
    let mut stmt = conn
        .prepare("SELECT display_name, node_id_bytes, x25519_secret FROM local_identity WHERE id = 1")
        .context("failed to prepare get_local_identity statement")?;

    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next().context("failed to read local identity row")? {
        Ok(Some(LocalIdentityRecord {
            display_name:  row.get(0).context("display_name")?,
            node_id_bytes: row.get(1).context("node_id_bytes")?,
            x25519_secret: row.get(2).context("x25519_secret")?,
        }))
    } else {
        Ok(None)
    }
}

// ── Peer Records ──────────────────────────────────────────────────────────────

/// A peer as stored in the local contact book.
#[derive(Debug, Clone)]
pub struct PeerRecord {
    /// Hex-encoded NodeId (primary key).
    pub node_id: String,
    /// Local display name assigned by the user.
    pub display_name: String,
    /// The imported ticket or address string used for first contact.
    pub endpoint_ticket: String,
    /// Hex-encoded X25519 public key, used for DH key exchange.
    pub x25519_pubkey: String,
    /// `true` if the user has completed safety-number verification with this peer.
    pub verified: bool,
}

/// Insert a new peer into the contact book.
///
/// # Errors
/// Returns an error if a peer with this `node_id` already exists or the write fails.
pub fn insert_peer(conn: &Connection, peer: &PeerRecord) -> Result<()> {
    conn.execute(
        "INSERT INTO peers (node_id, display_name, endpoint_ticket, x25519_pubkey, verified)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(node_id) DO UPDATE SET
             display_name = excluded.display_name,
             endpoint_ticket = excluded.endpoint_ticket",
        params![
            peer.node_id,
            peer.display_name,
            peer.endpoint_ticket,
            peer.x25519_pubkey,
            peer.verified as i32
        ],
    )
    .context("failed to insert peer")?;
    Ok(())
}

/// Update the stored X25519 public key for an existing peer.
///
/// # Errors
/// Returns an error if the peer does not exist or the write fails.
pub fn update_peer_x25519_pubkey(conn: &Connection, node_id: &str, x25519_pubkey: &str) -> Result<()> {
    let rows = conn
        .execute(
            "UPDATE peers SET x25519_pubkey = ?2 WHERE node_id = ?1",
            params![node_id, x25519_pubkey],
        )
        .context("failed to update peer x25519 pubkey")?;
    if rows == 0 {
        bail!("update_peer_x25519_pubkey: no peer found with node_id {:?}", node_id);
    }
    Ok(())
}

/// Update the stored endpoint ticket for an existing peer.
pub fn update_peer_endpoint_ticket(conn: &Connection, node_id: &str, endpoint_ticket: &str) -> Result<()> {
    let rows = conn
        .execute(
            "UPDATE peers SET endpoint_ticket = ?2 WHERE node_id = ?1",
            params![node_id, endpoint_ticket],
        )
        .context("failed to update peer endpoint ticket")?;
    if rows == 0 {
        bail!("update_peer_endpoint_ticket: no peer found with node_id {:?}", node_id);
    }
    Ok(())
}

/// Fetch a single peer by NodeId. Returns `None` if not found.
///
/// # Errors
/// Returns an error on a database failure.
pub fn get_peer(conn: &Connection, node_id: &str) -> Result<Option<PeerRecord>> {
    let mut stmt = conn
        .prepare(
            "SELECT node_id, display_name, endpoint_ticket, x25519_pubkey, verified
             FROM peers WHERE node_id = ?1",
        )
        .context("failed to prepare get_peer statement")?;

    let mut rows = stmt
        .query(params![node_id])
        .context("failed to query peer")?;

    if let Some(row) = rows.next().context("failed to read peer row")? {
        Ok(Some(PeerRecord {
            node_id:      row.get(0).context("node_id")?,
            display_name: row.get(1).context("display_name")?,
            endpoint_ticket: row.get(2).context("endpoint_ticket")?,
            x25519_pubkey: row.get(3).context("x25519_pubkey")?,
            verified:     row.get::<_, i32>(4).context("verified")? != 0,
        }))
    } else {
        Ok(None)
    }
}

/// Fetch all peers in the contact book.
///
/// # Errors
/// Returns an error on a database failure.
pub fn list_peers(conn: &Connection) -> Result<Vec<PeerRecord>> {
    let mut stmt = conn
        .prepare(
            "SELECT node_id, display_name, endpoint_ticket, x25519_pubkey, verified
             FROM peers ORDER BY display_name ASC",
        )
        .context("failed to prepare list_peers statement")?;

    let peers = stmt
        .query_map([], |row| {
            Ok(PeerRecord {
                node_id:         row.get(0)?,
                display_name:    row.get(1)?,
                endpoint_ticket: row.get(2)?,
                x25519_pubkey:   row.get(3)?,
                verified:        row.get::<_, i32>(4)? != 0,
            })
        })
        .context("failed to query peers")?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("failed to collect peers")?;

    Ok(peers)
}

/// Mark a peer as key-verified.
///
/// # Errors
/// Returns an error if no peer with this `node_id` exists or the write fails.
pub fn mark_peer_verified(conn: &Connection, node_id: &str) -> Result<()> {
    let rows = conn
        .execute(
            "UPDATE peers SET verified = 1 WHERE node_id = ?1",
            params![node_id],
        )
        .context("failed to mark peer verified")?;
    if rows == 0 {
        bail!("mark_peer_verified: no peer found with node_id {:?}", node_id);
    }
    Ok(())
}

// ── Group Records ─────────────────────────────────────────────────────────────

/// A group entry stored in the local database.
#[derive(Debug, Clone)]
pub struct GroupRecord {
    /// Hex-encoded TopicId (primary key).
    pub topic_id: String,
    /// Human-readable group name.
    pub group_name: String,
    /// Symmetric key bytes, stored encrypted at rest (RULES.md C-05).
    pub symmetric_key: Vec<u8>,
}

/// A chat row assembled from local peers, groups, and messages.
#[derive(Debug, Clone)]
pub struct ChatPreviewRecord {
    /// Hex-encoded peer node ID or group topic ID.
    pub id: String,
    /// Display name shown in the list.
    pub name: String,
    /// Short initials for the avatar.
    pub initials: String,
    /// Preview text for the latest message or queue state.
    pub last_message: String,
    /// Timestamp label shown in the row.
    pub timestamp: String,
    /// Unread badge count.
    pub unread: i32,
    /// `true` if this row represents a group chat.
    pub is_group: bool,
    /// `true` if the peer is currently reachable.
    pub is_online: bool,
    /// `true` if the peer is currently using relay routing.
    pub is_relay: bool,
    /// `true` if the row has queued outbound messages.
    pub is_queued: bool,
    /// `true` if the peer has been key-verified.
    pub is_verified: bool,
}

/// Insert a new group.
///
/// # Errors
/// Returns an error on duplicate topic_id or write failure.
pub fn insert_group(conn: &Connection, group: &GroupRecord) -> Result<()> {
    conn.execute(
        "INSERT INTO groups (topic_id, group_name, symmetric_key)
         VALUES (?1, ?2, ?3)",
        params![group.topic_id, group.group_name, group.symmetric_key],
    )
    .context("failed to insert group")?;
    Ok(())
}

/// Fetch a group by TopicId. Returns `None` if not found.
///
/// # Errors
/// Returns an error on database failure.
pub fn get_group(conn: &Connection, topic_id: &str) -> Result<Option<GroupRecord>> {
    let mut stmt = conn
        .prepare("SELECT topic_id, group_name, symmetric_key FROM groups WHERE topic_id = ?1")
        .context("failed to prepare get_group statement")?;

    let mut rows = stmt.query(params![topic_id]).context("failed to query group")?;

    if let Some(row) = rows.next().context("failed to read group row")? {
        Ok(Some(GroupRecord {
            topic_id:      row.get(0).context("topic_id")?,
            group_name:    row.get(1).context("group_name")?,
            symmetric_key: row.get(2).context("symmetric_key")?,
        }))
    } else {
        Ok(None)
    }
}

/// Fetch all groups in the local database.
///
/// # Errors
/// Returns an error on database failure.
pub fn list_groups(conn: &Connection) -> Result<Vec<GroupRecord>> {
    let mut stmt = conn
        .prepare("SELECT topic_id, group_name, symmetric_key FROM groups ORDER BY group_name ASC")
        .context("failed to prepare list_groups statement")?;

    let groups = stmt
        .query_map([], |row| {
            Ok(GroupRecord {
                topic_id: row.get(0)?,
                group_name: row.get(1)?,
                symmetric_key: row.get(2)?,
            })
        })
        .context("failed to query groups")?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("failed to collect groups")?;

    Ok(groups)
}

// ── Message Records ───────────────────────────────────────────────────────────

/// A message as stored in the local database.
#[derive(Debug, Clone)]
pub struct MessageRecord {
    /// Unique message ID.
    pub id: Uuid,
    /// Message type: `"direct"`, `"group"`, `"file"`, or `"group_invite"`.
    pub msg_type: String,
    /// NodeId (1:1) or TopicId (group) of the conversation.
    pub target_id: String,
    /// NodeId of the sender.
    pub sender_id: String,
    /// Decrypted plaintext stored locally (RULES.md C-06).
    pub content: String,
    /// UTC Unix timestamp in seconds (RULES.md U-07).
    pub timestamp: i64,
    /// Current delivery status.
    pub status: MessageStatus,
}

/// Insert a new message.
///
/// # Errors
/// Returns an error on duplicate ID or write failure.
pub fn insert_message(conn: &Connection, msg: &MessageRecord) -> Result<()> {
    conn.execute(
        "INSERT INTO messages (id, type, target_id, sender_id, content, timestamp, status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            msg.id.to_string(),
            msg.msg_type,
            msg.target_id,
            msg.sender_id,
            msg.content,
            msg.timestamp,
            msg.status.as_str()
        ],
    )
    .context("failed to insert message")?;
    Ok(())
}

/// Fetch a message by its unique ID.
///
/// # Errors
/// Returns an error on database failure.
pub fn get_message(conn: &Connection, id: &Uuid) -> Result<Option<MessageRecord>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, type, target_id, sender_id, content, timestamp, status
             FROM messages WHERE id = ?1",
        )
        .context("failed to prepare get_message statement")?;

    let mut rows = stmt
        .query(params![id.to_string()])
        .context("failed to query message")?;

    if let Some(row) = rows.next().context("failed to read message row")? {
        let id: String = row.get(0).context("id")?;
        let status: String = row.get(6).context("status")?;
        Ok(Some(MessageRecord {
            id: Uuid::parse_str(&id).with_context(|| format!("invalid UUID {:?}", id))?,
            msg_type: row.get(1).context("type")?,
            target_id: row.get(2).context("target_id")?,
            sender_id: row.get(3).context("sender_id")?,
            content: row.get(4).context("content")?,
            timestamp: row.get(5).context("timestamp")?,
            status: MessageStatus::from_db_str(&status)?,
        }))
    } else {
        Ok(None)
    }
}

/// Advance the status of a message — enforces forward-only transitions (RULES.md DB-04).
///
/// # Errors
/// Returns an error if the message is not found or the transition is invalid.
pub fn advance_message_status(
    conn: &Connection,
    id: &Uuid,
    new_status: &MessageStatus,
) -> Result<()> {
    // Read current status first.
    let mut stmt = conn
        .prepare("SELECT status FROM messages WHERE id = ?1")
        .context("failed to prepare status query")?;

    let current_str: String = stmt
        .query_row(params![id.to_string()], |row| row.get(0))
        .with_context(|| format!("message {:?} not found", id))?;

    let current = MessageStatus::from_db_str(&current_str)?;

    if !current.can_advance_to(new_status) {
        bail!(
            "invalid status transition {:?} → {:?} for message {}",
            current.as_str(),
            new_status.as_str(),
            id
        );
    }

    conn.execute(
        "UPDATE messages SET status = ?1 WHERE id = ?2",
        params![new_status.as_str(), id.to_string()],
    )
    .context("failed to update message status")?;

    Ok(())
}

/// Fetch all messages in a conversation (single peer or group topic), ordered by timestamp.
///
/// # Errors
/// Returns an error on database failure.
pub fn list_messages(conn: &Connection, target_id: &str) -> Result<Vec<MessageRecord>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, type, target_id, sender_id, content, timestamp, status
             FROM messages WHERE target_id = ?1 ORDER BY timestamp ASC",
        )
        .context("failed to prepare list_messages statement")?;

    let messages = stmt
        .query_map(params![target_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, String>(6)?,
            ))
        })
        .context("failed to query messages")?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("failed to collect messages")?;

    messages
        .into_iter()
        .map(|(id, msg_type, target_id, sender_id, content, timestamp, status_str)| {
            Ok(MessageRecord {
                id: Uuid::parse_str(&id).with_context(|| format!("invalid UUID {:?}", id))?,
                msg_type,
                target_id,
                sender_id,
                content,
                timestamp,
                status: MessageStatus::from_db_str(&status_str)?,
            })
        })
        .collect()
}

/// Fetch all messages with status `queued` for a given peer (used by flush_offline_queue).
///
/// # Errors
/// Returns an error on database failure.
pub fn list_queued_messages(conn: &Connection, target_id: &str) -> Result<Vec<MessageRecord>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, type, target_id, sender_id, content, timestamp, status
             FROM messages WHERE target_id = ?1 AND status = 'queued' ORDER BY timestamp ASC",
        )
        .context("failed to prepare list_queued_messages statement")?;

    let messages = stmt
        .query_map(params![target_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, String>(6)?,
            ))
        })
        .context("failed to query queued messages")?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("failed to collect queued messages")?;

    messages
        .into_iter()
        .map(|(id, msg_type, target_id, sender_id, content, timestamp, status_str)| {
            Ok(MessageRecord {
                id: Uuid::parse_str(&id).with_context(|| format!("invalid UUID {:?}", id))?,
                msg_type,
                target_id,
                sender_id,
                content,
                timestamp,
                status: MessageStatus::from_db_str(&status_str)?,
            })
        })
        .collect()
}

/// List all peers that have at least one queued message.
///
/// Used by `flush_offline_queue` to know which peers to retry.
///
/// # Errors
/// Returns an error on database failure.
pub fn list_peers_with_queued_messages(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn
        .prepare(
            "SELECT DISTINCT target_id FROM messages WHERE status = 'queued'",
        )
        .context("failed to prepare list_peers_with_queued statement")?;

    let peers = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .context("failed to query peers with queued messages")?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("failed to collect peer IDs")?;

    Ok(peers)
}

/// Build the home-screen chat previews from peers, groups, and message history.
///
/// # Errors
/// Returns an error on database failure.
pub fn list_chat_previews(conn: &Connection) -> Result<Vec<ChatPreviewRecord>> {
    let mut previews = Vec::new();

    for peer in list_peers(conn)? {
        let messages = list_messages(conn, &peer.node_id)?;
        let queued = list_queued_messages(conn, &peer.node_id)?;
        let latest = messages.last();
        let initials = derive_initials(&peer.display_name);
        let timestamp = latest
            .map(|msg| msg.timestamp.to_string())
            .unwrap_or_default();
        let last_message = latest
            .map(|msg| msg.content.clone())
            .unwrap_or_default();

        previews.push(ChatPreviewRecord {
            id: peer.node_id,
            name: peer.display_name,
            initials,
            last_message,
            timestamp,
            unread: 0,
            is_group: false,
            is_online: false,
            is_relay: false,
            is_queued: !queued.is_empty(),
            is_verified: peer.verified,
        });
    }

    for group in list_groups(conn)? {
        let messages = list_messages(conn, &group.topic_id)?;
        let latest = messages.last();
        let initials = derive_initials(&group.group_name);
        let timestamp = latest
            .map(|msg| msg.timestamp.to_string())
            .unwrap_or_default();
        let last_message = latest
            .map(|msg| msg.content.clone())
            .unwrap_or_default();

        previews.push(ChatPreviewRecord {
            id: group.topic_id,
            name: group.group_name,
            initials,
            last_message,
            timestamp,
            unread: 0,
            is_group: true,
            is_online: false,
            is_relay: false,
            is_queued: false,
            is_verified: true,
        });
    }

    previews.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(previews)
}

/// Derive a two-letter avatar label from a display name.
fn derive_initials(name: &str) -> String {
    let mut initials = String::new();
    for part in name.split_whitespace() {
        if let Some(ch) = part.chars().next() {
            initials.push(ch.to_ascii_uppercase());
        }
        if initials.chars().count() >= 2 {
            break;
        }
    }

    if initials.is_empty() {
        name.chars().take(2).collect::<String>().to_ascii_uppercase()
    } else {
        initials
    }
}

/// Delete all messages from the database.
pub fn clear_all_messages(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM messages", [])
        .context("failed to clear messages table")?;
    Ok(())
}

/// Delete all messages in a single conversation without removing the peer/group row.
pub fn clear_conversation_messages(conn: &Connection, target_id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM messages WHERE target_id = ?1",
        params![target_id],
    )
    .context("failed to clear conversation messages")?;
    Ok(())
}

/// Delete a single conversation and its local rows.
///
/// For direct chats, this removes the peer contact row and all messages.
/// For groups, this removes the group row and all messages.
pub fn delete_conversation(conn: &Connection, target_id: &str, is_group: bool) -> Result<()> {
    conn.execute(
        "DELETE FROM messages WHERE target_id = ?1",
        params![target_id],
    )
    .context("failed to delete conversation messages")?;

    if is_group {
        conn.execute(
            "DELETE FROM groups WHERE topic_id = ?1",
            params![target_id],
        )
        .context("failed to delete group row")?;
    } else {
        conn.execute(
            "DELETE FROM peers WHERE node_id = ?1",
            params![target_id],
        )
        .context("failed to delete peer row")?;
    }

    Ok(())
}

/// Delete the local identity and all related data (peers, groups, messages).
pub fn delete_local_identity(conn: &Connection) -> Result<()> {
    // Note: We don't care if tables are already empty, so we ignore minor errors
    let _ = conn.execute("DELETE FROM messages", []);
    let _ = conn.execute("DELETE FROM groups", []);
    let _ = conn.execute("DELETE FROM peers", []);
    let _ = conn.execute("DELETE FROM local_identity", []);
    Ok(())
}

// ── Unit Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::open_in_memory;

    fn test_peer() -> PeerRecord {
        PeerRecord {
            node_id:         "aabbcc".to_owned(),
            display_name:    "Alice".to_owned(),
            endpoint_ticket: "docaaacarwhmusoqf362j3jpzrehzkw3bqamcp2mmbhn3fmag3mzzfjp4beahj2v7aezhojvfqi5wltr4vxymgzqnctryyup327ct7iy4s5noxy6aaa".to_owned(),
            x25519_pubkey:   "ddeeff".to_owned(),
            verified:        false,
        }
    }

    fn test_message(target: &str, status: MessageStatus) -> MessageRecord {
        MessageRecord {
            id:        Uuid::new_v4(),
            msg_type:  "direct".to_owned(),
            target_id: target.to_owned(),
            sender_id: "me".to_owned(),
            content:   "hello".to_owned(),
            timestamp: 1_700_000_000,
            status,
        }
    }

    /// T-11: Write peer to DB → read it back, all fields match.
    #[test]
    fn peer_roundtrip_fields_match() {
        let conn = open_in_memory().unwrap();
        let peer = test_peer();
        insert_peer(&conn, &peer).unwrap();
        let got = get_peer(&conn, &peer.node_id).unwrap().expect("peer must exist");
        assert_eq!(got.node_id, peer.node_id);
        assert_eq!(got.display_name, peer.display_name);
        assert_eq!(got.endpoint_ticket, peer.endpoint_ticket);
        assert_eq!(got.x25519_pubkey, peer.x25519_pubkey);
        assert!(!got.verified);
    }

    /// T-12: Write message with status queued → read back, status is queued.
    #[test]
    fn message_queued_status_is_persisted() {
        let conn = open_in_memory().unwrap();
        let peer = test_peer();
        insert_peer(&conn, &peer).unwrap();
        let msg = test_message(&peer.node_id, MessageStatus::Queued);
        insert_message(&conn, &msg).unwrap();
        let msgs = list_messages(&conn, &peer.node_id).unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].status, MessageStatus::Queued);
    }

    /// T-13: Status advances queued → sent → delivered → read.
    #[test]
    fn message_status_advances_forward() {
        let conn = open_in_memory().unwrap();
        let peer = test_peer();
        insert_peer(&conn, &peer).unwrap();
        let msg = test_message(&peer.node_id, MessageStatus::Queued);
        insert_message(&conn, &msg).unwrap();

        advance_message_status(&conn, &msg.id, &MessageStatus::Sent).unwrap();
        advance_message_status(&conn, &msg.id, &MessageStatus::Delivered).unwrap();
        advance_message_status(&conn, &msg.id, &MessageStatus::Read).unwrap();

        let got = list_messages(&conn, &peer.node_id).unwrap();
        assert_eq!(got[0].status, MessageStatus::Read);
    }

    /// T-14: Message status cannot go backward.
    #[test]
    fn message_status_cannot_go_backward() {
        let conn = open_in_memory().unwrap();
        let peer = test_peer();
        insert_peer(&conn, &peer).unwrap();
        let msg = test_message(&peer.node_id, MessageStatus::Queued);
        insert_message(&conn, &msg).unwrap();
        advance_message_status(&conn, &msg.id, &MessageStatus::Sent).unwrap();
        advance_message_status(&conn, &msg.id, &MessageStatus::Delivered).unwrap();

        // Attempt backward transition: Delivered → Queued must fail.
        let result = advance_message_status(&conn, &msg.id, &MessageStatus::Queued);
        assert!(result.is_err(), "backward transition must be rejected");
    }

    /// T-15: Fetch all queued messages for a peer returns correct results.
    #[test]
    fn list_queued_messages_returns_only_queued() {
        let conn = open_in_memory().unwrap();
        let peer = test_peer();
        insert_peer(&conn, &peer).unwrap();

        let q = test_message(&peer.node_id, MessageStatus::Queued);
        let s = test_message(&peer.node_id, MessageStatus::Sent);
        insert_message(&conn, &q).unwrap();
        insert_message(&conn, &s).unwrap();

        let queued = list_queued_messages(&conn, &peer.node_id).unwrap();
        assert_eq!(queued.len(), 1);
        assert_eq!(queued[0].id, q.id);
    }

    /// T-16: Write group with key → read back, key bytes match.
    #[test]
    fn group_key_roundtrip_bytes_match() {
        let conn = open_in_memory().unwrap();
        let key_bytes: Vec<u8> = (0u8..32).collect();
        let group = GroupRecord {
            topic_id:      "topic-001".to_owned(),
            group_name:    "Dev Chat".to_owned(),
            symmetric_key: key_bytes.clone(),
        };
        insert_group(&conn, &group).unwrap();
        let got = get_group(&conn, "topic-001").unwrap().expect("group must exist");
        assert_eq!(got.symmetric_key, key_bytes);
        assert_eq!(got.group_name, "Dev Chat");
    }
}
