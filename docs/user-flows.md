# NodeChat User Flows

## Purpose

This document explains how a user currently moves through NodeChat. It describes the implemented app behavior, not an ideal future version.

## 1. Create A Local Identity

For a first-time user, the app begins with identity setup.

Current flow:

1. The user opens the app and enters a display name.
2. The user may also set a PIN or password for local app protection.
3. The app creates the local identity and stores the required local data.
4. The user can then continue into the main app experience.

Outcome:

- the user now has a local identity
- the app can show a connection ticket for peer onboarding
- the app can restore this identity in later sessions

## 2. Unlock The App

If local protection is enabled, the app requires the user to unlock before entering the main interface.

Current flow:

1. The app opens to the lock screen.
2. The user enters the configured PIN.
3. On success, the app unlocks and returns the user to the main experience.

Outcome:

- local identity data remains protected behind the app lock flow
- the user resumes their saved chats and settings

## 3. Add A Contact

NodeChat allows a user to add another peer through a connection ticket or peer identifier.

Current flow:

1. The user opens the add-peer screen.
2. The user pastes a peer ticket or node identifier.
3. The app attempts to add that peer to local contacts.
4. On success, the app shows a visible success notice and returns from the add screen.

Outcome:

- the contact is saved locally
- the contact can appear in the direct conversation list
- the app can begin direct communication work with that peer

## 4. Start A Direct Conversation

After a contact is added, the user can open a direct chat.

Current flow:

1. The user selects a saved contact from the contact or chat list.
2. The app opens the conversation view for that peer.
3. The user sends a message through the composer.
4. The backend handles transmission and updates message state as progress becomes available.

The app distinguishes between:

- peer presence or transport availability
- secure-session readiness
- manual trust verification

Outcome:

- the user can exchange direct messages
- the conversation history is stored locally
- message status can move through queued, sent, delivered, and read

## 5. Verify A Contact

Verification is a trust action, not an automatic transport event.

Current flow:

1. The user opens a contact’s info screen.
2. The user toggles verification for that contact.
3. The app updates the saved trust state for that peer.

Outcome:

- the contact is marked as verified or unverified by user choice
- this trust decision remains separate from handshake or session establishment

## 6. Create A Group

NodeChat supports creating a shared group conversation from inside the app.

Current flow:

1. The user opens the create-group screen.
2. The user enters a group name and optional description.
3. The app creates the group locally and prepares a group key and topic.
4. The user selects contacts to invite.
5. Invitations are sent through existing direct messaging paths.

Outcome:

- the group is saved locally
- invited peers can receive the group invitation through direct messages
- the creator can open the new group conversation in the chat interface

## 7. Accept A Group Invitation

Group membership is currently driven by invitation handling inside direct conversations.

Current flow:

1. A user receives a group invitation message in a direct chat.
2. The user opens or accepts that invitation from the message content.
3. The app stores the group locally if it is not already saved.
4. The app joins the group transport using the invitation data.
5. The user is taken into the group conversation after the join succeeds.

Outcome:

- the group becomes part of the user’s saved conversations
- invitation messages for that group are marked as read
- the user can participate in the group conversation

## 8. Retry Queued Work

If a direct peer is unavailable, messages or invite payloads may remain queued until communication can continue.

Current flow:

1. The user sees queued message state in the interface.
2. The user triggers a retry path from the app.
3. The app refreshes backend state and attempts to continue pending communication work.

Outcome:

- queued work can resume when the required peer connection becomes available
- the UI can update as status changes are received

## 9. Clear History Or Remove Conversations

The app supports destructive actions through confirmation flows.

Current flows include:

- clearing message history
- deleting a direct conversation
- leaving or deleting a group conversation

These actions pass through confirmation controls before the backend applies them.

Outcome:

- destructive actions are not executed silently
- when local protection is enabled, the app can require confirmation PIN input

## 10. Reset The Local Identity

The app also supports a full identity reset.

Current flow:

1. The user opens settings.
2. The user chooses the identity reset action.
3. The app shows a destructive confirmation step.
4. After confirmation, the local identity and related local data are removed.
5. The app returns to the initial setup state.

Outcome:

- the previous local identity is removed from the device
- the app returns to first-run style onboarding

## Notes On Current Behavior

These flows should be presented carefully during defense:

- direct communication readiness is not the same as manual verification
- group onboarding currently depends on invitation exchange through direct conversations
- local confirmations and notices are part of the real UX, not just placeholder design elements

The next companion documents should explain what the app offers in feature terms and where its current limits still exist.
