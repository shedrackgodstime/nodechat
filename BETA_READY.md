# NodeChat Beta Readiness Checklist

This file tracks the remaining work needed to reach a usable beta release.
The goal is not feature completeness. The goal is a stable, understandable,
and low-risk app that can be tested on desktop and Android without constant
breakage.

## Beta Definition

NodeChat is beta-ready when:

- A fresh install can create an identity and reach the chat list.
- A returning user can reopen the app without losing local state.
- Direct peer chat works reliably on desktop and Android.
- Queued messages survive offline periods and resend automatically.
- The app makes offline / queued / reconnecting state obvious in the UI.
- Android does not look broken when the user backgrounds and foregrounds the app.
- Crashes and runtime errors are visible, copyable, and actionable.

## Current Status

- Identity onboarding exists.
- Direct peer discovery and handshake bootstrap exist.
- Direct transport has been proven with the probe.
- Chat list, direct chat, and queue UI are already wired.
- Runtime error display exists.
- Android permissions and lifecycle hooks exist.
- True Android foreground service and persistent notification are not done yet.

## Must Have Before Beta

### 1. Startup and recovery

- [ ] Confirm fresh install startup works on desktop.
- [ ] Confirm fresh install startup works on Android.
- [ ] Confirm returning-user startup restores identity, contacts, and chats.
- [ ] Confirm local SQLite migrations are safe on upgrade installs.
- [ ] Confirm app launch never wipes queued messages.
- [ ] Confirm app launch restores the selected chat state cleanly.

### 2. Direct peer reliability

- [ ] Confirm add-peer -> handshake -> send-message works repeatedly.
- [ ] Confirm offline peer queueing works without message loss.
- [ ] Confirm queued messages flush automatically after reconnect.
- [ ] Confirm retry-now works from the chat header.
- [ ] Confirm delivery/read receipts do not break decryption.
- [ ] Confirm reconnect after app background/foreground works.

### 3. Offline and presence behavior

- [ ] Keep peer offline detection fast without false positives.
- [ ] Keep peer online/offline labels accurate in the chat list.
- [ ] Keep queued badges visible when outbound messages are waiting.
- [ ] Keep reconnecting / handshake pending states visible in the chat header.
- [ ] Avoid aggressive reconnect loops for long-offline peers.

### 4. Android stability

- [ ] Verify the app stays stable when the screen rotates.
- [ ] Verify the app survives normal Android pause/resume transitions.
- [ ] Verify backgrounding the app does not immediately look like a transport failure.
- [ ] Decide whether beta requires a true foreground service or only graceful background handling.
- [ ] If required for beta, add a foreground service plus persistent notification.

### 5. Error handling

- [ ] Keep the runtime error tray visible and copyable.
- [ ] Make sure backend failures always surface in the UI.
- [ ] Make sure panics are captured and shown.
- [ ] Remove any leftover debug binaries or debug-only entry points.
- [ ] Reduce noisy warnings that do not help beta testing.

### 6. UI clarity

- [ ] Keep chat rows readable on mobile.
- [ ] Keep queued/offline/verified state obvious in list rows.
- [ ] Keep chat headers clear about connection and handshake state.
- [ ] Keep delete-conversation working for single chats and groups.
- [ ] Keep the mobile navigation flow predictable.

## Should Have For Beta

- [ ] Clearer message status presentation: sent, delivered, read.
- [ ] Better chat list ordering by last activity.
- [ ] A small "last seen" label for peers.
- [ ] Cleaner timestamp formatting in lists and threads.
- [ ] A smaller mobile error banner or icon-based close control.
- [ ] Better empty states for chats and contacts.

## After Beta

- [ ] Real Android foreground service and persistent notification if not done before beta.
- [ ] Group gossip path parity and polish.
- [ ] File transfer.
- [ ] Read/delivery receipt refinements beyond the current basics.
- [ ] Contact verification workflow improvements.
- [ ] Better reconnect backoff tuning.
- [ ] Background push or always-on presence improvements.

## Suggested Order

1. Startup and recovery.
2. Direct peer reliability.
3. Offline and presence behavior.
4. Android stability.
5. Error handling.
6. UI clarity.

That sequence keeps the work focused on what will actually make the app
shippable instead of expanding scope.
