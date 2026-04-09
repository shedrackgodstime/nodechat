# NodeChat New Frontend-Ready Contract

This document defines the target contract for `nodechat-new` before any UI/backend wiring work begins.

The purpose is to make the next phase predictable:

- Slint renders backend state
- Slint emits user intents
- Rust owns domain state and domain actions
- mock data lives in Rust only

This is the contract we should refactor toward.

## 1. Core Rule

The UI must not invent domain state.

That means:

- no hard-coded chat datasets in Slint
- no hard-coded contact datasets in Slint
- no local fake conversation switching in Slint
- no local message thread replacement in Slint
- no local identity truth in Slint

Slint may still own:

- view routing state
- modal open/close state
- text input draft state
- temporary form values before submit
- keyboard and safe-area layout state

## 2. Canonical Identifier Model

We need one stable identifier policy before wiring anything.

## 2.1 Contacts

Contacts should expose:

- `contact_id`: identifies the local contact record
- `peer_id`: identifies the remote node / person

Rule:

- the UI may show both
- backend mutations against a contact record use `contact_id`
- messaging and verification actions that target the peer use `peer_id` or `conversation_id` depending on context

## 2.2 Conversations

Conversations should expose:

- `conversation_id`: the stable thread identifier used to load and append messages
- `kind`: `direct` or `group`

Rule:

- chat list rows open by `conversation_id`
- message send actions target `conversation_id`
- the UI should not guess a thread id from a contact row

## 2.3 Groups

Groups should expose:

- `conversation_id`: stable group thread id
- `topic_id`: optional transport-level group identifier if different

If `conversation_id` and `topic_id` are the same in the mock backend, keep one public UI field and hide the transport detail inside the backend.

## 2.4 Recommended Direction

For the frontend-facing contract, prefer these names:

- `contact_id`
- `peer_id`
- `conversation_id`

Do not overload `id` to mean different things on different screens.

## 3. Frontend-Facing State Model

The frontend should consume a single backend-owned app snapshot shape.

Recommended root state:

- `identity`
- `app_flags`
- `chat_list`
- `contact_list`
- `group_candidates`
- `active_conversation`
- `active_messages`
- `debug_feed`

## 3.1 Identity

Identity state should be one coherent backend-owned object:

- `display_name`
- `initials`
- `peer_id`
- `endpoint_ticket`
- `is_locked`
- `has_identity`

This should replace scattered root-level identity fields where possible.

## 3.2 App Flags

These are backend-derived flags the UI may display:

- `direct_peer_count`
- `relay_peer_count`
- `is_offline`

These are still backend-owned because they describe app/network state, not presentation state.

## 3.3 Chat List Item

Each chat row should expose:

- `conversation_id`
- `kind`
- `title`
- `initials`
- `last_message`
- `timestamp`
- `unread_count`
- `is_online`
- `is_relay`
- `is_verified`
- `is_session_ready`
- `has_queued_messages`

This replaces the current split between Rust `ChatPreview` and Slint `ChatPreview`.

## 3.4 Contact List Item

Each contact row should expose:

- `contact_id`
- `peer_id`
- `display_name`
- `initials`
- `is_online`
- `is_relay`
- `is_verified`
- `is_session_ready`
- `direct_conversation_id`

Important:

The contact list must expose the direct conversation entry point explicitly.
That removes the current guesswork where the UI passes `contact.id` into conversation loading.

## 3.5 Active Conversation

The selected conversation should expose:

- `conversation_id`
- `kind`
- `title`
- `initials`
- `peer_id` for direct conversations, empty for group if needed
- `ticket`
- `is_online`
- `is_relay`
- `is_verified`
- `is_session_ready`
- `connection_stage`
- `member_count`
- `return_screen`

This should become the single source of truth for the chat header.

## 3.6 Message Item

The frontend message shape should support both direct and group messages without local Slint invention.

Recommended fields:

- `message_id`
- `conversation_id`
- `sender_name`
- `text`
- `timestamp`
- `is_outgoing`
- `is_system`
- `status`
- `kind`

Optional metadata block:

- `invite_group_name`
- `invite_topic_id`
- `invite_key`
- `invite_is_joined`
- `is_ephemeral`
- `ttl_seconds`

Message `kind` should describe rendering behavior:

- `standard`
- `system`
- `group_invite`

That is cleaner than asking Slint to infer everything from many booleans.

## 3.7 Group Candidate Item

If group creation is part of the next UI phase, expose:

- `contact_id`
- `display_name`
- `initials`
- `is_online`
- `is_selected`

## 3.8 Debug Feed

If diagnostics remain in scope, expose:

- ordered log lines
- optional severity later

This is backend-owned even if the screen is not built yet.

## 4. Frontend Command Surface

These are domain intents the UI should be allowed to emit.

## 4.1 Conversation and Messaging

- `LoadConversation { conversation_id }`
- `SendMessage { conversation_id, plaintext }`
- `RetryQueuedMessage { conversation_id, message_id }` if retry remains supported
- `DeleteConversation { conversation_id, confirmation_pin? }` if supported

Recommended simplification:

Use one `SendMessage` command instead of separate direct/group send commands at the frontend boundary.
The backend already knows the conversation kind.

## 4.2 Contacts and Groups

- `AddContact { ticket_or_peer_id }`
- `CreateGroup { name, member_contact_ids }`
- `ToggleGroupCandidate { contact_id }`
- `OpenDirectConversation { contact_id }`

Recommendation:

`OpenDirectConversation` may be handled entirely in backend or translated into `LoadConversation` by first looking up `direct_conversation_id`.
The UI should not do that lookup itself if the contract already exposes `direct_conversation_id`.

## 4.3 Identity and Security

- `CreateIdentity { display_name, pin }`
- `FinalizeIdentity`
- `UnlockApp { pin }`
- `ChangePassword { current_pin, new_pin }`
- `UpdateDisplayName { display_name }`
- `ResetIdentity { confirmation_pin }`

## 4.4 Conversation / Trust Actions

- `SetVerification { peer_id, verified }`
- `AcceptGroupInvite { conversation_id, topic_id, invite_key }`

## 4.5 Data / Maintenance Actions

- `ClearMessageHistory { scope, confirmation_pin? }`
- `Refresh`

Clipboard actions should not become backend commands unless platform constraints force that.

## 5. Backend Event Surface

The UI should receive state changes in a way that minimizes local reconstruction.

Recommended event surface:

- `SnapshotReady(AppSnapshot)`
- `IdentityUpdated(IdentityView)`
- `ChatListUpdated(Vec<ChatListItem>)`
- `ContactListUpdated(Vec<ContactListItem>)`
- `ConversationUpdated(ConversationView)`
- `MessageListReplaced { conversation_id, messages }`
- `MessageAppended { conversation_id, message }`
- `GroupCandidatesUpdated(Vec<GroupCandidateItem>)`
- `DebugFeedUpdated(Vec<String>)`
- `StatusNotice(String)`
- `UserError(String)`

Recommendation:

Prefer event names that describe UI-consumable state rather than backend internals.

## 6. What Should Remain UI-Local

These should stay out of the backend contract:

- `current_screen`
- desktop/mobile layout mode
- safe area insets
- keyboard height
- draft text in composer
- temporary modal visibility
- temporary PIN field contents
- transient selection highlight

These are view concerns, not domain concerns.

## 7. What Must Be Removed From Slint

Before wiring, remove these responsibilities from `ui/app.slint`:

- hard-coded `chats`
- hard-coded `contacts`
- hard-coded `active-messages`
- hard-coded identity defaults used as app truth
- local `open-chat` mock thread switching
- local `open-contact` conversation synthesis

The root can keep fallback empty values, but not real app datasets.

## 8. Action Classification

Not every current Slint callback should become a backend command.

## 8.1 UI-Only Actions

These may remain purely local routing/presentation actions:

- `open-settings`
- `open-contacts`
- `go-back`
- `open-contact-info`
- `open-group-info`
- showing and dismissing confirm modals

These actions do not need backend round-trips unless the target screen depends on newly fetched backend state.

## 8.2 Backend Actions

These should be emitted to Rust:

- conversation loading
- sending messages
- adding contacts
- creating groups
- changing identity state
- changing password
- deleting conversations
- resetting identity
- verification changes
- invite acceptance
- clearing history

## 8.3 Platform Actions

These may be handled by a thin UI/platform layer:

- copy peer id
- copy ticket
- maybe share ticket

These are not core backend domain operations.

## 9. Recommended Refactor Sequence

The next implementation phase should happen in this order:

1. Update Rust contract types to the frontend-ready model
2. Update mock backend data to match the new contract exactly
3. Add explicit mapping between Rust models and generated Slint types
4. Strip domain mock data out of `ui/app.slint`
5. Keep only empty/default rendering state in Slint
6. Wire Slint callbacks to Rust commands
7. Apply Rust events back into root properties
8. Remove old direct/group split where the frontend no longer needs it

## 10. Definition Of Contract Ready

The contract is ready when all of the following are true:

- every visible screen gets its data from backend-owned state or view-local state, never both
- every supported domain action has a Rust command path
- every backend update the UI needs has an event/state path
- contact opening does not depend on ambiguous identifiers
- Slint does not contain hard-coded app datasets beyond harmless empty defaults
- replacing mock backend data with real data later does not require redesigning Slint state ownership

That is the target state before full wiring begins.
