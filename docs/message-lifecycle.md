# NodeChat Message Lifecycle

## Purpose

This document explains how NodeChat currently handles message progression. It focuses on the visible states used by the app and the backend events that move a message from one stage to another.

The current message states are:

- queued
- sent
- delivered
- read

## 1. Queued

A message begins in the queued state when it has been created locally but has not yet completed transport transmission.

This can happen when:

- a peer is not currently connected
- a secure direct session is not yet ready
- a group has no active neighbors at that moment

Why this matters:

- the app does not pretend every send action succeeds immediately
- queued state is a real part of the communication model
- the queue gives the app a way to resume work when connectivity improves

## 2. Sent

A queued message becomes sent after the backend successfully pushes it onto the current transport path.

In the current app:

- direct messages become sent after successful direct transport transmission
- group messages become sent after successful group broadcast transmission

This means `sent` should be interpreted as:

- “the local app has transmitted the message successfully”

It should not be described as proof that the recipient has already opened or confirmed the message.

## 3. Delivered

Delivered means the app has stronger evidence that a message reached the receiving side.

For direct conversations:

- when a direct message is received, the receiving app stores it locally
- the receiver sends a receipt back to the sender
- when that receipt is processed, the sender-side message state advances to delivered

For incoming messages:

- if the receiving conversation is not currently open, the message is stored as delivered

This means `delivered` is stronger than `sent`, but it is still different from `read`.

## 4. Read

Read means the app treats the message as seen in the active conversation context.

Current behavior:

- if a direct message arrives while that conversation is open, it is stored as read immediately
- the receiver sends a receipt reflecting read state in that situation
- for accepted group invitation messages, matching invite records can also be advanced to read
- for group messages, if the group conversation is currently active, incoming messages are stored as read

This means `read` reflects current app visibility rather than a broad cross-device attention model.

## 5. Direct Message Path

The current direct-message lifecycle is:

1. the user sends a message
2. the message is stored locally in queued state
3. the backend checks whether the peer is connected and whether secure-session material is available
4. if needed, the app triggers handshake or reconnection work
5. once transmission succeeds, the message becomes sent
6. when a receipt returns from the receiving side, the message becomes delivered
7. if the receiver had that conversation open at receipt time, the message can effectively be acknowledged as read

This is the best concise explanation for direct-message state handling in the current app.

## 6. Group Message Path

The group-message lifecycle is slightly different because group traffic does not use the same direct receipt model.

Current group flow:

1. the user sends a group message
2. the message is stored locally in queued state
3. the app checks whether the joined group currently has active neighbors
4. if group broadcast succeeds, the message becomes sent
5. when another participant receives the message, the receiving app stores it as delivered or read depending on whether that group conversation is currently open

Important difference:

- direct messages use explicit receipts
- group messaging currently relies on group transport behavior and local receive handling rather than the same direct receipt exchange

## 7. Invite And Share Messages

NodeChat also transports structured message content such as:

- contact shares
- group invitations

These still pass through the message system and therefore participate in the same general lifecycle model:

- they may be queued
- they may be transmitted
- they may be received and stored
- invitation-related records may later be marked as read when the invitation is accepted

## 8. Retry Behavior

Queued work is not abandoned immediately. The app has retry behavior tied to connectivity and refresh paths.

In practical terms:

- a peer connection event can trigger queued direct work
- a group neighbor event can trigger queued group work
- the user can also trigger retry-related refresh behavior from the UI

This is why queued state should be explained as a temporary working state, not a terminal error state.

## 9. How To Explain The States During Defense

The most accurate short explanation is:

- `queued` means the message is stored locally and waiting for a viable transport path
- `sent` means the local app has transmitted it
- `delivered` means the receiving side has acknowledged receipt or stored it as received
- `read` means the message reached an active-view context in the receiving app

That explanation matches the current NodeChat implementation closely and is strong enough for academic defense.
