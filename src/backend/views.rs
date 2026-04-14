use anyhow::Result;
use chrono::{TimeZone, Utc};

use crate::contract::{
    AppFlags, AppInfoView, AppSnapshot, ChatListItem, ContactListItem,
    ConversationKind, ConversationView, GroupCandidateItem, IdentityView,
    MessageItem, MessageKind, MessageStatus,
};
use crate::storage::queries::{self, derive_initials, MessageRecord};
use super::RealBackend;
use super::utils::{format_hms, format_date_label};

impl RealBackend {
    /// Build the full application snapshot for the initial UI load.
    pub fn snapshot(&self) -> AppSnapshot {
        self.build_snapshot().unwrap_or_else(|e| {
            tracing::error!("CRITICAL: Snapshot generation failed: {}", e);
            AppSnapshot {
                identity:             IdentityView::empty(),
                app_info:             AppInfoView::current(),
                app_flags:            AppFlags { direct_peer_count: 0, relay_peer_count: 0, is_offline: true },
                chat_list:            vec![],
                contact_list:         vec![],
                group_candidates:     vec![],
                active_conversation:  ConversationView::empty(ConversationKind::Direct),
                active_messages:      vec![],
                debug_feed:           vec![format!("[ERROR] critical snapshot failure: {}", e)],
            }
        })
    }

    pub(super) fn build_snapshot(&self) -> Result<AppSnapshot> {
        let identity = self.build_identity_view().unwrap_or_else(|e| {
            tracing::error!("Snapshot: failed to build identity view: {}", e);
            IdentityView::empty()
        });

        let chat_list = self.build_chat_list().unwrap_or_else(|e| {
            tracing::error!("Snapshot: failed to build chat list: {}", e);
            vec![]
        });

        let contact_list = self.build_contact_list().unwrap_or_else(|e| {
            tracing::error!("Snapshot: failed to build contact list: {}", e);
            vec![]
        });

        let group_candidates = self.build_group_candidates().unwrap_or_else(|e| {
            tracing::error!("Snapshot: failed to build group candidates: {}", e);
            vec![]
        });

        Ok(AppSnapshot {
            identity,
            app_info:            AppInfoView::current(),
            app_flags:           AppFlags { direct_peer_count: 0, relay_peer_count: 0, is_offline: true },
            chat_list,
            contact_list,
            group_candidates,
            active_conversation: ConversationView::empty(ConversationKind::Direct),
            active_messages:     vec![],
            debug_feed:          vec![],
        })
    }

    pub(super) fn build_identity_view(&self) -> Result<IdentityView> {
        match queries::get_local_identity(&self.conn)? {
            None => Ok(IdentityView::empty()),
            Some(id) => Ok(IdentityView {
                display_name:    id.display_name.clone(),
                initials:        derive_initials(&id.display_name),
                peer_id:         id.node_id_hex,
                endpoint_ticket: id.endpoint_ticket,
                is_locked:       !id.pin_hash.is_empty(),
                has_identity:    true,
                has_password:    !id.pin_hash.is_empty(),
            }),
        }
    }

    pub(super) fn build_chat_list(&self) -> Result<Vec<ChatListItem>> {
        let previews = queries::list_chat_previews(&self.conn, &self.local_node_id)?;
        Ok(previews.into_iter().map(|p| {
            let neighbor_count = if p.is_group { self.network.group_neighbor_count(&p.id) } else { 0 };
            let is_online = if p.is_group { neighbor_count > 0 } else { self.network.has_connection(&p.id) };
            ChatListItem {
                conversation_id:          p.id,
                kind:                     if p.is_group { ConversationKind::Group } else { ConversationKind::Direct },
                title:                    p.title,
                initials:                 p.initials,
                last_message:             if !p.is_session_ready && p.last_message.is_empty() { "Waiting for handshake...".to_string() } else { p.last_message },
                last_message_status:      p.last_message_status,
                is_last_message_outgoing: p.is_outgoing,
                timestamp:                format_hms(p.timestamp),
                member_count:             if p.is_group { (neighbor_count + 1) as i32 } else { 0 },     
                unread_count:             p.unread_count,     
                is_online,
                is_relay:                 false, 
                is_verified:              p.is_verified,
                is_session_ready:         if p.is_group { neighbor_count > 0 } else { p.is_session_ready },
                has_queued_messages:      p.has_queued,
            }
        }).collect())
    }

    pub(super) fn build_contact_list(&self) -> Result<Vec<ContactListItem>> {
        let peers = queries::list_peers(&self.conn)?;
        Ok(peers.into_iter().map(|p| {
            let is_online = self.network.has_connection(&p.node_id);
            ContactListItem {
                contact_id:              p.node_id.clone(),
                peer_id:                 p.node_id.clone(),
                display_name:            p.display_name.clone(),
                initials:                derive_initials(&p.display_name),
                is_online,
                is_relay:                false, 
                is_verified:             p.verified,
                is_session_ready:        !p.x25519_pubkey.is_empty(),
                direct_conversation_id:  p.node_id, 
            }
        }).collect())
    }

    pub(super) fn build_group_candidates(&self) -> Result<Vec<GroupCandidateItem>> {
        let peers = queries::list_peers(&self.conn)?;
        Ok(peers.into_iter().map(|p| {
            let is_online = self.network.has_connection(&p.node_id);
            GroupCandidateItem {
                contact_id:   p.node_id.clone(),
                display_name: p.display_name.clone(),
                initials:     derive_initials(&p.display_name),
                is_selected:  self.selected_candidates.contains(&p.node_id),
                is_online,
            }
        }).collect())
    }

    pub(super) fn build_conversation_view(&self, conversation_id: &str) -> Result<ConversationView> {
        // Group check first
        if let Some(group) = queries::get_group(&self.conn, conversation_id)? {
            let neighbor_count = self.network.group_neighbor_count(conversation_id);
            return Ok(ConversationView {
                conversation_id:  group.topic_id.clone(),
                kind:             ConversationKind::Group,
                title:            group.group_name.clone(),
                initials:         derive_initials(&group.group_name),
                peer_id:          String::new(),
                ticket:           String::new(),
                is_online:        neighbor_count > 0,
                is_relay:         false,
                is_verified:      true,
                is_session_ready: neighbor_count > 0,
                connection_stage: if neighbor_count > 0 { "Swarm Active".to_string() } else { "Connecting to Swarm...".to_string() },
                member_count:     (neighbor_count + 1) as i32,
                return_screen:    0,
            });
        }
        // Direct peer
        if let Some(peer) = queries::get_peer(&self.conn, conversation_id)? {
            let is_online = self.network.has_connection(&peer.node_id);
            let is_session_ready = !peer.x25519_pubkey.is_empty();
            let connection_stage = if is_online {
                if peer.verified { "Secure P2P session active" } else { "Handshaking..." }
            } else {
                "Peer offline"
            }.to_string();

            return Ok(ConversationView {
                conversation_id:  peer.node_id.clone(),
                kind:             ConversationKind::Direct,
                title:            peer.display_name.clone(),
                initials:         derive_initials(&peer.display_name),
                peer_id:          peer.node_id.clone(),
                ticket:           peer.endpoint_ticket,
                is_online,
                is_relay:         false,
                is_verified:      peer.verified,
                is_session_ready,
                connection_stage,
                member_count:     1,
                return_screen:    0,
            });
        }
        Ok(ConversationView::empty(ConversationKind::Direct))
    }

    pub(super) fn build_message_items(&self, target_id: &str) -> Result<Vec<MessageItem>> {
        let messages = queries::list_messages(&self.conn, target_id)?;
        let mut items = Vec::new();
        let mut last_date = None;

        for record in messages {
            let datetime = Utc.timestamp_opt(record.timestamp, 0).single().unwrap_or_else(|| Utc.timestamp_opt(0,0).unwrap());
            let date = datetime.date_naive();
            
            if Some(date) != last_date {
                items.push(MessageItem {
                    message_id:        format!("date-{}", record.timestamp),
                    conversation_id:   target_id.to_string(),
                    sender_name:       String::new(),
                    text:              format_date_label(record.timestamp),
                    timestamp:         String::new(),
                    received_timestamp: String::new(),
                    is_delayed:        false,
                    is_outgoing:       false,
                    is_system:         true,
                    status:            MessageStatus::Delivered,
                    kind:              MessageKind::System,
                    invite_group_name: String::new(),
                    invite_topic_id:   String::new(),
                    invite_key:        String::new(),
                    invite_is_joined:  false,
                });
                last_date = Some(date);
            }
            items.push(self.to_message_item(&record)?);
        }
        Ok(items)
    }

    pub(super) fn to_message_item(&self, record: &MessageRecord) -> Result<MessageItem> {
        let is_outgoing = record.sender_id == self.local_node_id;

        let sender_name = if is_outgoing {
            self.local_display_name.clone()
        } else {
            match queries::get_peer(&self.conn, &record.sender_id)? {
                Some(peer) => peer.display_name,
                None    => record.sender_id.chars().take(8).collect(),
            }
        };

        let invite_is_joined = !record.invite_topic_id.is_empty()
            && queries::group_exists(&self.conn, &record.invite_topic_id)?;

        let kind = match record.kind.as_str() {
            "group_invite" => MessageKind::GroupInvite,
            "system"       => MessageKind::System,
            "contact_share" => MessageKind::ContactShare,
            _              => MessageKind::Standard,
        };

        Ok(MessageItem {
            message_id:       record.id.to_string(),
            conversation_id:  record.target_id.clone(),
            sender_name,
            text:             record.content.clone(),
            timestamp:        format_hms(record.timestamp),
            received_timestamp: format_hms(record.received_at),
            is_delayed:       record.received_at > 0 && (record.received_at > record.timestamp + 300),
            is_outgoing,
            is_system:        record.kind == "system",
            status:           record.status,
            kind,
            invite_group_name: record.invite_group_name.clone(),
            invite_topic_id:   record.invite_topic_id.clone(),
            invite_key:        record.invite_key.clone(),
            invite_is_joined,
        })
    }
}
