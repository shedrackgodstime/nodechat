# NodeChat New Implementation Order

This is the practical build sequence for `nodechat-new`.
The goal is to keep the foundation stable, avoid layout sprawl, and make the project easy to continue later.

## 1. Lock The Core Contract

Build and keep stable:

- `Command` and `AppEvent` types
- snapshot model
- chat/contact/message data shapes
- bridge/runtime boundary

Why this comes first:

- the UI must not depend on internal storage or networking details
- the backend must not depend on view structure
- every future screen needs the same data contract

**Reference:** [`nodechat-new/src/contract.rs`](/home/kristency/Projects/nodechat/nodechat-new/src/contract.rs), [`nodechat-new/src/bridge.rs`](/home/kristency/Projects/nodechat/nodechat-new/src/bridge.rs), [`nodechat-new/src/mock_backend.rs`](/home/kristency/Projects/nodechat/nodechat-new/src/mock_backend.rs)

## 2. Build The App Shell

Create the shell before any feature screens:

- app frame
- responsive breakpoint logic
- safe-area handling
- one navigation controller
- one primary scroll surface per page

The shell should decide:

- mobile vs desktop
- list vs detail panel
- keyboard offset behavior
- back navigation

**Reference:** [`nodechat-new/ui/app.slint`](/home/kristency/Projects/nodechat/nodechat-new/ui/app.slint), [`nodechat-new/ui/theme.slint`](/home/kristency/Projects/nodechat/nodechat-new/ui/theme.slint)

## 3. Finalize Shared UI Tokens

Before adding more screens, keep one source of truth for:

- color
- spacing
- typography
- radius
- card styling
- divider styling

This keeps the visual language consistent across mobile and desktop.

**Reference:** [`nodechat-new/ui/theme.slint`](/home/kristency/Projects/nodechat/nodechat-new/ui/theme.slint)

## 4. Stabilize Reusable Components

Build components that can be reused across pages:

- list row
- message bubble
- composer / input bar
- header shell
- action row
- confirmation modal

These should be generic enough to work for chats, contacts, and settings.

**Reference:** [`nodechat-new/ui/components/chat_row.slint`](/home/kristency/Projects/nodechat/nodechat-new/ui/components/chat_row.slint), [`nodechat-new/ui/components/message_bubble.slint`](/home/kristency/Projects/nodechat/nodechat-new/ui/components/message_bubble.slint), [`nodechat-new/ui/components/composer.slint`](/home/kristency/Projects/nodechat/nodechat-new/ui/components/composer.slint)

## 5. Build The Core Screens In Order

### 5.1 Chat List

This is the main home screen.

Focus on:

- chat preview rows
- unread counts
- empty state
- mobile/desktop list behavior

**Reference:** [`ui/screens/chat_list.slint`](/home/kristency/Projects/nodechat/ui/screens/chat_list.slint)

### 5.2 Chat View

This is the main working screen.

Focus on:

- message stream
- keyboard-safe composer
- direct vs group conversation modes
- one scroll region for messages

**Reference:** [`ui/screens/chat_view.slint`](/home/kristency/Projects/nodechat/ui/screens/chat_view.slint), [`ui/components/chat_input.slint`](/home/kristency/Projects/nodechat/ui/components/chat_input.slint)

### 5.3 Contacts

Add this after the chat shell is stable.

Focus on:

- directory rows
- quick actions
- contact detail entry points

**Reference:** [`ui/screens/contacts.slint`](/home/kristency/Projects/nodechat/ui/screens/contacts.slint)

### 5.4 Settings

Add this once identity and chat shells are stable.

Focus on:

- profile card
- copy actions
- password and danger zone behavior

**Reference:** [`ui/screens/settings.slint`](/home/kristency/Projects/nodechat/ui/screens/settings.slint)

## 6. Add Secondary Detail Screens

Only build these after the main pages are stable:

- peer info
- conversation actions
- add contact
- create group
- edit name
- change password
- select members
- diagnostics

These should reuse the same shell and component patterns, not introduce new layout rules.

**Reference:** [`ui/screens/contact_details.slint`](/home/kristency/Projects/nodechat/ui/screens/contact_details.slint), [`ui/screens/conversation_actions.slint`](/home/kristency/Projects/nodechat/ui/screens/conversation_actions.slint), [`ui/screens/add_contact.slint`](/home/kristency/Projects/nodechat/ui/screens/add_contact.slint), [`ui/screens/create_group.slint`](/home/kristency/Projects/nodechat/ui/screens/create_group.slint), [`ui/screens/edit_name.slint`](/home/kristency/Projects/nodechat/ui/screens/edit_name.slint), [`ui/screens/change_password.slint`](/home/kristency/Projects/nodechat/ui/screens/change_password.slint), [`ui/screens/select_members.slint`](/home/kristency/Projects/nodechat/ui/screens/select_members.slint), [`ui/screens/diagnostics.slint`](/home/kristency/Projects/nodechat/ui/screens/diagnostics.slint)

## 7. Handle Mobile Keyboard Behavior Early

This is not a polishing task. It is a core layout task.

Rules:

- keep the composer visible when the keyboard opens
- avoid nested scroll regions around editable content
- keep focused inputs inside one scrollable ancestor
- make enter/send behavior predictable
- make back navigation work with the keyboard open

**Reference:** [`nodechat-new/SLINT_UI_DOCS_NOTES.md`](/home/kristency/Projects/nodechat/nodechat-new/SLINT_UI_DOCS_NOTES.md)

## 8. Only Then Reconnect Real Networking

After the shell is stable:

- replace mock backend behavior gradually
- keep the bridge contract unchanged
- wire real storage and networking behind the same events
- do not move UI code into backend modules

This keeps the migration small and prevents regressions in presentation.

**Reference:** [`nodechat-new/src/mock_backend.rs`](/home/kristency/Projects/nodechat/nodechat-new/src/mock_backend.rs), [`nodechat-new/src/bridge.rs`](/home/kristency/Projects/nodechat/nodechat-new/src/bridge.rs)

## 9. Definition Of Done For Each Screen

Every screen should meet these rules before moving on:

- works on mobile and desktop
- has one clear primary action
- has consistent spacing and typography
- does not create a new layout pattern unless absolutely necessary
- uses shared tokens and shared components
- handles empty states cleanly
- keeps input focus and keyboard behavior sane

## 10. The Rule To Keep Us Honest

If a new screen starts feeling complicated, stop and extract a reusable shell or component.
Do not keep stacking layout logic inside the screen itself.

That is how we keep `nodechat-new` clean enough to explain, present, and maintain.

