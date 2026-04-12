# NodeChat UI Reduction Checklist

Goal: reduce repeated UI structure so the chat and detail screens feel like one system, not a set of separate pages.

## Priority 1: Shared Header

- [ ] Create a reusable top bar component for the main app pages.
- [ ] Use it in `ui/screens/chat_list.slint`.
- [ ] Use it in `ui/screens/contacts.slint`.
- [ ] Use it in `ui/screens/contact_details.slint`.
- [ ] Use it in `ui/screens/conversation_actions.slint`.
- [ ] Keep the header flexible enough for:
  - [ ] back button
  - [ ] title
  - [ ] subtitle
  - [ ] right-side action button
  - [ ] optional icon/avatar area

## Priority 2: Shared Conversation Header

- [ ] Create one reusable conversation header block for chat-related pages.
- [ ] Use it in `ui/screens/chat_view.slint`.
- [ ] Use it in `ui/screens/conversation_actions.slint`.
- [ ] Support both direct and group mode through props, not duplicated layout.
- [ ] Keep the current `active-conversation.kind` as the source of truth.

## Priority 3: Shared Action Rows

- [ ] Create one reusable action-row component.
- [ ] Use it for:
  - [ ] `Open Chat`
  - [ ] `Clear Chat History`
  - [ ] `Delete Contact`
  - [ ] `Exit Group`
  - [ ] `Retry Queued Messages`
- [ ] Keep row styling, spacing, and hit area consistent across pages.

## Priority 4: Shared Page Shell

- [ ] Create a reusable shell for detail-style pages.
- [ ] Reuse the same outer structure for:
  - [ ] `ContactsScreen`
  - [ ] `ContactDetailsScreen`
  - [ ] `ConversationActionsScreen`
- [ ] Keep the shell responsible for:
  - [ ] background
  - [ ] safe-area padding
  - [ ] centered content width
  - [ ] section spacing

## Priority 5: Shared Conversation State Helpers

- [ ] Reduce repeated manual assignment to `active-conversation` fields.
- [ ] Add one helper path for setting a direct conversation.
- [ ] Add one helper path for setting a group conversation.
- [ ] Keep `active-conversation-messages` as the single thread model.

## Priority 6: Smaller Cleanup

- [ ] Remove any leftover direct/group-specific layout branches that only differ visually.
- [ ] Keep behavior changes separate from pure visual reduction work.
- [ ] Re-run `cargo check` after each major UI reduction.

## Order To Do

1. Shared Header
2. Shared Conversation Header
3. Shared Action Rows
4. Shared Page Shell
5. Shared Conversation State Helpers

