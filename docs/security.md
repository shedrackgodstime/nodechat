# NodeChat Security

## Purpose

This document explains the current security model of NodeChat based on the implemented application. It focuses on what the app secures today, how trust is represented, and where the present boundaries still exist.

## Security Goals

NodeChat is designed around a few practical security goals:

- keep user identity local to the app instance
- establish secure communication material between direct peers
- protect direct and group message payloads in transit
- make user trust a separate decision from transport success

These goals are important because they shape both the code and the user-facing behavior of the app.

## 1. Local Identity Ownership

Each NodeChat installation maintains its own local identity. The application stores the local identity in its local database, including the information needed to participate in peer communication.

This matters because the app does not depend on a central account system to create or manage identity during normal use.

Current security meaning:

- identity is local to the device and app instance
- the app can optionally protect access with a PIN or password flow
- identity reset is a local destructive action handled inside the app

## 2. Direct Session Establishment

For direct conversations, NodeChat uses a handshake process to exchange the information needed for secure communication.

From the current implementation:

- peers exchange handshake frames
- the handshake includes an X25519 public key and connection details
- the app derives a shared secret from local and remote key material
- that shared secret is then used for direct message encryption

The important defense point is this:

- a handshake creates secure-session readiness
- it does not automatically mark the peer as trusted

That distinction is now reflected in both the backend logic and the UI.

## 3. Direct Message Protection

When a direct peer session is ready, NodeChat encrypts direct message payloads before transport using a symmetric key derived from peer key exchange.

The current implementation uses:

- X25519-based shared-secret derivation
- SHA-256 as a key-derivation step on the shared secret
- ChaCha20-Poly1305 for authenticated encryption

This gives the app two important security properties for protected direct messages:

- confidentiality of the message payload
- integrity checking, so tampered ciphertext fails authentication

## 4. Group Message Protection

For groups, NodeChat uses a different model from direct messaging.

In the current app:

- a group is created with a symmetric group key
- invitation data carries the information needed for invited users to join
- group traffic is encrypted using that shared group key
- the app stores the group key locally for joined groups

This allows group participants to exchange encrypted group messages once they are part of the same group conversation.

## 5. Verification And Trust

NodeChat treats verification as a user trust action, not as a network side effect.

That means:

- transport-level connection is not the same as trust
- handshake completion is not the same as trust
- verification is a separate saved user decision

This is one of the most important things to explain clearly during defense, because it shows that the system distinguishes between:

- “can communicate securely”
- “is trusted by the user”

Those are related, but they are not identical.

## 6. Local Storage Boundaries

NodeChat stores application state locally in SQLite. This includes:

- local identity data
- saved contacts
- saved group records
- stored message history

From a security perspective, this means:

- the app preserves state across sessions
- trust and conversation history remain locally available
- local data handling is part of the overall security story, not separate from it

At the same time, local persistence also means the project should describe storage honestly and avoid implying that every piece of local app data is hidden behind a broader platform-grade secure enclave or recovery system.

## 7. Message-State And Security Signaling

NodeChat also uses visible UI state to communicate security-relevant conditions.

Examples include:

- handshake or connection-stage indicators
- session-readiness cues
- manual verification state
- queued, sent, delivered, and read message progression

These indicators matter because users need to understand not just whether a chat exists, but what stage communication is in.

## 8. Current Security Boundaries

NodeChat’s present security model should be described carefully.

What the app does today:

- supports local identity ownership
- establishes secure-session material for direct peers
- encrypts direct and group payloads in transit using implemented key material
- separates trust verification from handshake success

What should be described more modestly:

- advanced trust-verification ceremony
- recovery and portability workflows
- broader production-grade operational guarantees
- full commercial-scale resilience claims

## How To Defend The Security Model

The strongest professional explanation is:

- NodeChat secures communication through local identity, key exchange, and encrypted transport
- the app distinguishes secure session establishment from user trust
- the design is intentional and appropriate for the project’s current scope

That framing is accurate to the app and strong enough for academic defense.
