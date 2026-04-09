# NodeChat New Frontend / Backend Audit

This document defines what the Slint layer should own, what the Rust backend should own, what is currently mocked in the UI, and what must be aligned before wiring.

The goal is simple:

- Slint should become a clean rendering layer plus UI interaction layer.
- Rust should become the source of truth for app state and domain actions.
- After this cleanup, UI work should mostly stay in `ui/` and backend work should mostly stay in `src/`.

## 1. Current Problem

Right now the project has three different layers mixed together:

- visual layout and interaction in Slint
- app state mocked directly in Slint
- a separate typed mock backend in Rust

This creates duplicate truth.

Examples:

- `ui/app.slint` contains demo chats, contacts, messages, and conversation state
- `src/mock_backend.rs` contains a second mock app state
- `src/ui.rs` does not yet connect the Rust runtime to the Slint root

That means the UI is not yet truly driven by backend state.

## 2. What Is Currently Mocked In Slint

The following data is currently hard-coded or locally mutated in `ui/app.slint` and should not remain UI-owned long term.

### Root state defaults

File: `ui/app.slint`

- `current-screen`
- `has-identity`
- `is-locked`
- `display-name`
- `initials`
- `endpoint-ticket`
- `chats`
- `contacts`
- `active-messages`
- `active-conversation`
- `direct-peers`
- `relay-peers`
- `is-offline`
- confirm modal state fields

### Local mock conversation switching

File: `ui/app.slint`

The `open-chat(chat)` callback currently:

- calls `load-conversation(chat.id, chat.is-group)`
- also mutates `active-conversation` locally
- also replaces `active-messages` with hard-coded per-chat mock data
- also changes `current-screen`

This is a temporary prototype behavior. In the cleaned architecture, opening a chat should emit an intent and then render backend state.

### Contacts screen local handoff

File: `ui/app.slint`

The `open-contact(contact)` branch currently:

- calls `load-conversation(contact.id, false)`
- also mutates `active-conversation` locally
- also changes `current-screen`

This also duplicates backend responsibility.

## 3. What Is Currently Mocked In Rust

Rust already has a separate mock domain model in:

- `src/contract.rs`
- `src/mock_backend.rs`
- `src/bridge.rs`

The Rust mock backend already models:

- identity
- chats
- contacts
- group candidates
- active conversation
- direct messages
- group messages
- debug logs

This is the correct direction. Mock domain state belongs in Rust, not in Slint.

## 4. Slint Surface Inventory

These are the current UI-level callbacks and state surfaces exposed by Slint.

### Root callbacks in `ui/app.slint`

- `open-settings`
- `open-contacts`
- `add-contact`
- `open-chat(ChatPreview)`
- `send-message(string)`
- `send-group-message(string)`
- `load-conversation(string, bool)`
- `open-contact-info()`
- `open-group-info()`
- `accept-group-invite(string, string, string)`
- `retry-queued()`
- `request-delete-conversation()`
- `confirm-modal-confirmed(string, string)`
- `edit-display-name()`
- `copy-node-id()`
- `change-password()`
- `open-debug-console()`
- `request-clear-history()`
- `request-reset-identity()`

### Screen callbacks

`ui/screens/settings_screen.slint`

- `edit-display-name()`
- `copy-node-id()`
- `change-password()`
- `open-debug-console()`
- `request-clear-history()`
- `request-reset-identity()`

`ui/screens/contacts_screen.slint`

- `add-contact()`
- `new-group()`
- `open-contact(ContactData)`

`ui/screens/chat_view.slint`

- `go-back()`
- `send-message(string)`
- `open-contact-info()`
- `open-invite(string, string, string, bool)`
- `retry-queued()`
- `request-delete-conversation()`

## 5. Rust Backend Surface Inventory

Current Rust commands in `src/contract.rs`:

- `Refresh`
- `LoadConversation`
- `SendDirectMessage`
- `SendGroupMessage`
- `ToggleVerified`
- `ToggleGroupMemberSelection`
- `AddContact`
- `CreateGroup`
- `CreateIdentity`
- `FinaliseIdentity`
- `UnlockApp`
- `ChangePassword`

Current Rust events in `src/contract.rs`:

- `SnapshotReady`
- `ChatsUpdated`
- `ContactsUpdated`
- `ConversationLoaded`
- `DirectMessageAppended`
- `GroupMessageAppended`
- `IdentityUpdated`
- `Status`
- `Error`

## 6. Mismatch Map

## 6.1 Slint Has Actions Without Rust Support

These UI actions do not yet have a clean backend command/event path:

- `open-contact-info`
- `open-group-info`
- `accept-group-invite`
- `retry-queued`
- `request-delete-conversation`
- `confirm-modal-confirmed` as a domain action dispatcher
- `edit-display-name`
- `copy-node-id`
- `open-debug-console`
- `request-clear-history`
- `request-reset-identity`

Notes:

- `copy-node-id` may remain a UI/platform action if it only copies existing state to clipboard.
- `open-contact-info` and `open-group-info` may be routing-only actions if they only open local detail screens.
- destructive actions and data mutations should be backend commands.

## 6.2 Rust Has State Not Fully Represented In Slint

The Rust contract contains fields not fully surfaced in the Slint structs:

### Chat preview mismatch

Rust `ChatPreview` includes:

- `is_queued`
- `is_session_ready`

Slint `ChatPreview` currently omits both.

### Conversation state mismatch

Rust `ConversationState` includes:

- `ticket`
- `member_count`

Slint `ConversationContext` currently omits both.

### Identity mismatch

Rust `IdentityCard` includes:

- `display_name`
- `initials`
- `node_id`
- `endpoint_ticket`
- `is_locked`

Slint root currently spreads identity across separate root properties and does not model a single identity object.

### Snapshot mismatch

Rust `AppSnapshot` includes:

- `group_candidates`
- `debug_logs`

The current Slint root does not expose a full state path for these.

## 6.3 Identifier Mismatch

This is one of the most important blockers.

Current state:

- backend direct chats use chat ids like `7f1f3b2a...`
- contacts use ids like `contact-ada`
- contacts also carry `node_id`
- UI contact opening currently passes `contact.id` to `load-conversation`

This is inconsistent.

We must choose one canonical identifier for direct conversation loading.

Recommended rule:

- `ContactData.id` should identify the contact record
- `ContactData.node-id` should identify the peer
- `ChatPreview.id` should identify the conversation/thread
- direct-conversation loading must use one stable backend-defined thread id

If direct thread id and peer id are intentionally the same, make that explicit everywhere.

## 7. Ownership Rules For The Clean Architecture

## 7.1 Slint Should Own Only UI State

These should stay in Slint:

- current visible screen
- local modal visibility
- local form draft text
- local checkbox/toggle presentation state when not committed yet
- safe area values
- virtual keyboard layout offsets
- temporary selection/highlight state

Rule:

If the value affects business logic, persistence, transport, identity, contacts, or messages, it should not be UI-owned.

## 7.2 Rust Should Own App State

These should be backend-owned:

- identity
- lock/unlock state
- chats list
- contacts list
- group candidate list
- active conversation
- message history
- delivery state
- verification/session state
- add contact
- create group
- send message
- conversation loading
- destructive actions
- password changes
- debug log stream

## 7.3 Shared Contract Layer

The contract should define:

- the data the UI receives
- the commands the UI may send
- the events/state updates the backend emits

The contract must be complete enough that Slint does not invent domain state locally.

## 8. What Should Happen To Existing UI Mock Data

### Must be removed from Slint and sourced from Rust

- demo chats
- demo contacts
- demo messages
- demo active conversation
- demo identity values
- backend-derived network counters if they matter beyond pure decoration

### May remain in Slint as purely local defaults

- default screen index
- modal open/close booleans
- composer draft
- temporary PIN input text inside modal
- safe area and keyboard values

## 9. Recommended Cleanup Order

Do not wire first.

Use this order:

1. Audit and freeze ownership rules
2. Normalize identifiers and shared structs
3. Expand or reduce the Rust contract until it exactly matches supported UI actions
4. Remove domain mock data from `ui/app.slint`
5. Keep only presentation-local state in Slint
6. Add Rust-to-Slint mapping functions
7. Wire callbacks from Slint to Rust commands
8. Apply backend events/snapshots back into Slint properties
9. Remove any leftover duplicate local mutation paths

## 10. Immediate Refactor Target

Before any callback wiring, the project should reach this state:

- one backend-owned mock state source in Rust
- zero hard-coded conversation/message datasets in Slint
- one agreed identifier model
- one agreed contract for every supported screen action
- one list of actions that are UI-local only

Once that is done, future UI work becomes much cleaner:

- layout changes happen in Slint
- domain changes happen in Rust
- replacing mock data with real data later stays mostly inside Rust

That is the correct foundation for this project.
