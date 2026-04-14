# NodeChat Architecture

## Purpose

This document gives a concise architecture view of NodeChat based on the implemented application. It is meant to support technical defense and future documentation work without drifting into older placeholder narrative.

## High-Level Structure

NodeChat is organized as a layered application with five main parts:

- Slint user interface
- UI-to-backend bridge
- backend coordination layer
- peer-to-peer transport layer
- local storage and cryptographic support

These layers work together to keep the UI responsive while networking, persistence, and message processing happen in the background.

## 1. User Interface Layer

The UI is built with Slint and is responsible for:

- screen layout and navigation
- user interaction handling
- rendering chats, contacts, groups, and settings
- showing state such as connection status, notices, and message progress

The UI does not directly own the application logic. Instead, it emits actions and receives updated state from the backend side of the app.

Relevant modules:

- `src/ui.rs`
- `src/ui_models.rs`
- `ui/`

## 2. UI Bridge Layer

The bridge sits between Slint’s synchronous world and the asynchronous backend/runtime world.

Its responsibilities are:

- receive UI commands
- forward them into backend processing
- collect backend events
- feed those events back into the UI event loop
- mirror selected logs into the in-app diagnostics console

This layer is important because it allows the app to keep a responsive interface while background tasks continue running.

Relevant module:

- `src/bridge.rs`

## 3. Backend Coordination Layer

The backend is the main application coordinator.

Its responsibilities include:

- command handling
- network-event handling
- building UI-facing views and snapshots
- managing active conversation context
- coordinating message transmission and retry logic
- keeping storage, transport, and UI state aligned

In practice, the backend is where most app behavior is decided.

Relevant modules:

- `src/backend/mod.rs`
- `src/backend/commands.rs`
- `src/backend/events.rs`
- `src/backend/ops.rs`
- `src/backend/views.rs`

## 4. Peer-To-Peer Transport Layer

The transport layer manages communication between peers and across group topics.

It handles:

- direct peer connectivity
- group subscription and broadcast
- wire-frame encoding and decoding
- network event generation for the backend

NodeChat uses:

- direct transport for one-to-one peer communication
- group transport for shared group conversations

Relevant modules:

- `src/p2p/mod.rs`
- `src/p2p/direct.rs`
- `src/p2p/group.rs`
- `src/p2p/protocol.rs`

## 5. Storage Layer

The storage layer provides local persistence through SQLite.

It stores:

- the local identity
- saved peers
- saved groups
- message history and message states

This layer is central to the app because NodeChat is not a stateless demo. It restores local state between sessions and gives the rest of the system durable application data.

Relevant modules:

- `src/storage/mod.rs`
- `src/storage/queries.rs`

## 6. Cryptographic Support

NodeChat includes a dedicated cryptographic utility layer used by direct and group messaging paths.

Its role is to provide:

- shared-secret derivation for direct messaging
- symmetric group-key generation
- authenticated encryption and decryption helpers

Relevant module:

- `src/crypto.rs`

## Runtime Flow

At runtime, the app behaves roughly like this:

1. the UI starts and creates the runtime
2. the bridge starts channels and the asynchronous backend worker
3. the backend opens local storage and initializes transport if an identity exists
4. the backend sends an initial application snapshot to the UI
5. UI actions are converted into commands
6. backend command handling updates storage, transport, or both
7. backend events are converted into UI model updates

This command-event pattern is one of the core structural ideas in the app.

## Data Flow Model

The most useful technical summary is:

- UI emits commands
- backend processes commands
- storage and transport produce state changes
- backend emits application events
- UI applies those events as updated visible state

That flow helps explain why the app remains understandable even though it combines networking, persistence, and interface logic.

## Key Architectural Strengths

The current architecture is strong in a few important ways:

- the UI is separated from backend logic
- transport and persistence are coordinated through a central backend layer
- message and conversation state are modeled explicitly
- security-sensitive concepts such as session readiness and verification are represented separately

These qualities make the project easier to defend as a real software system, not just a visual prototype.

## Current Architectural Boundaries

The architecture should still be presented within its real scope.

It is:

- suitable for a serious final-year project
- strong enough to demonstrate decentralized messaging structure
- already organized around clear modules and app state flow

It should not yet be described as a finalized large-scale production architecture with every operational concern fully solved.
