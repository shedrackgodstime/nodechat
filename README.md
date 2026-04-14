# NodeChat

NodeChat is a peer-to-peer messaging application built in Rust with a Slint user interface. It is developed as a final-year school project and focuses on local identity ownership, direct messaging, group messaging, encrypted transport, and clear application-level state handling.

The project should be understood as a serious academic prototype with real application behavior. It is not presented as a finished commercial messaging platform.

## Overview

NodeChat is designed to show how a messaging application can operate around peer-to-peer communication, local persistence, and security-aware interaction design without depending on a traditional central messaging workflow.

The current implementation already behaves like a real app:

- users create and keep a local identity
- peers connect through a shareable connection ticket
- direct and group conversations are stored locally
- the interface reflects connection state, trust state, and message progress

## What NodeChat Does

The current app supports:

- local identity creation and optional app unlock protection
- direct peer-to-peer conversations
- group conversations over peer-to-peer group transport
- local storage for contacts, groups, identity, and message history
- visible message states such as `queued`, `sent`, `delivered`, and `read`
- manual contact verification as a separate trust action
- in-app notices, status indicators, and confirmation flows

One of the important design points in the current app is that secure-session readiness and manual trust verification are treated as different things. A peer can be ready for secure communication without being automatically marked as trusted.

## Project Positioning

NodeChat is intended to demonstrate how a decentralized chat application can be structured as a complete app rather than as an isolated networking experiment.

The project combines:

- user interface design
- local persistence
- peer-to-peer communication
- message lifecycle handling
- security-oriented interaction design

That combination is what makes it suitable for project defense and for future extension into a broader product and documentation story.

## Design Principles

The current project direction is guided by a few clear principles:

- the app defines the story first, then the docs follow it
- user trust is treated separately from transport success
- local state and message progress should be visible and understandable
- claims should stay aligned with implemented behavior

These principles matter because they keep the project easier to explain, defend, and extend.

## Technology Stack

NodeChat is currently built with:

- Rust
- Slint
- Tokio
- Iroh
- Iroh Gossip
- SQLite via `rusqlite`
- `x25519-dalek`
- `chacha20poly1305`

## Getting Started

Clone the repository and run the desktop app:

```bash
git clone https://github.com/shedrackgodstime/nodechat.git
cd nodechat
cargo run
```

For a local verification pass:

```bash
cargo check
cargo test
```

If you want the broader project context before going deeper into the code, start with the documentation index linked below.

## Project Structure

```text
nodechat/
├── src/            Rust application code
├── ui/             Slint screens and shared UI components
├── assets/         Icons and image assets
├── docs/           Current project documentation
└── site/           Placeholder site that will later be driven by the docs
```

## Documentation

The current documentation is organized around the implemented app, not around placeholder claims.

Start here:

- [Documentation Index](./docs/index.md)
- [Overview](./docs/overview.md)
- [Features](./docs/features.md)
- [User Flows](./docs/user-flows.md)
- [Limitations](./docs/limitations.md)
- [Security](./docs/security.md)
- [Architecture](./docs/architecture.md)

Older architecture and archive material still exists in the repository, but the newer app-based docs should now be treated as the primary source of truth.

## Contributing

If you want to contribute, read:

- [Contributing Guide](./docs/contributing.md)
- [Rules](./docs/RULES.md)
- [Agent Guidance](./docs/AGENT.md)

Contributions should stay aligned with the implemented app, the current documentation direction, and the project’s professional tone.

## Current Scope

NodeChat currently presents itself as:

- a peer-to-peer messaging application
- a final-year academic project
- a working software system with real UI, storage, transport, and messaging behavior

It should not currently be described as a finished large-scale consumer platform.

That scope is intentional. The project is strongest when it is described precisely and defended from the behavior the app actually implements today.

## Repository Links

- Website: <https://nodechat.pages.dev>
- Repository: <https://github.com/shedrackgodstime/nodechat>
