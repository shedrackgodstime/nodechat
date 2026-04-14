# NodeChat Overview

## What NodeChat Is

NodeChat is a peer-to-peer messaging application built as a final-year school project. It focuses on direct communication between participants, local identity ownership, encrypted message transport, and a desktop-style user experience built with Rust and Slint.

The project is designed to show how a chat application can operate without relying on a traditional central messaging server for everyday conversation flow. Each user runs their own app instance, keeps their own local identity, and exchanges data with other peers through the network layer built into the application.

## What The App Does Today

In its current form, NodeChat supports:

- local identity creation and app unlock flow
- direct peer-to-peer conversations
- group conversations over a peer-to-peer group transport
- local persistence for identity, contacts, groups, and message history
- visible message-state handling for queued, sent, delivered, and read updates
- manual contact verification as a separate trust action
- in-app status and notice feedback for important operations

The app already distinguishes between secure-session readiness and manual trust verification. A successful handshake establishes session material for communication, while verification remains a user-controlled decision about trust.

## Core Concepts

### Local Identity

Each installation owns a local identity. That identity includes the information needed to participate in direct communication and to present a shareable connection ticket to other users.

### Contact And Conversation Model

NodeChat supports two main conversation types:

- direct conversations between two peers
- group conversations shared across multiple participants

Direct conversations depend on peer connection and secure-session establishment. Group conversations use a shared group topic and group transport to exchange messages among subscribed participants.

### Secure Session And Verification

NodeChat treats these as different concepts:

- a secure session means the application has completed the required handshake work to communicate securely with a peer
- verification means the user has explicitly marked that peer as trusted

This separation is important both for system clarity and for project defense. The app does not present automatic transport success as the same thing as manual trust.

### Local Storage

The application stores its working data locally. This includes the local identity, saved contacts, saved groups, and conversation history needed to restore the app state and continue previous interactions.

### Message State Tracking

NodeChat tracks message progression through user-visible states such as:

- queued
- sent
- delivered
- read

These states help communicate delivery progress and also make the app easier to explain during evaluation and demonstration.

## Current Scope

NodeChat currently presents itself as a serious prototype or academic project with real application behavior, not as a finished commercial messaging platform.

Its present scope is to demonstrate:

- decentralized communication between peers
- local ownership of identity and chat data
- direct and group messaging flows
- secure transport concepts and trust controls
- a usable interface for common messaging tasks

## Current Limitations

The app should be described carefully and honestly. At this stage:

- it is not positioned as a production-ready consumer messenger
- some advanced trust, recovery, and operational workflows remain limited
- the future website and full documentation set are still being built from the implemented app
- some deeper platform, deployment, and long-term scaling concerns are outside the current project scope

These limitations do not weaken the project. They define its actual maturity level and help keep the documentation credible.

## Why This Project Matters

NodeChat is valuable as a final-year project because it combines interface design, local persistence, peer-to-peer networking, message-state handling, and security-oriented thinking in one application. It is not only a UI exercise and not only a networking experiment. It is a complete app-level study of how decentralized messaging can be structured and presented.

## What To Read Next

The next documents should expand this overview from the implemented app:

- `docs/features.md` for product capabilities
- `docs/user-flows.md` for how a user moves through the app
- `docs/limitations.md` for precise current boundaries
