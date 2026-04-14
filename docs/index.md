# NodeChat Documentation Index

## Purpose

This index is the entry point for the current NodeChat documentation set. It is designed to help both project reviewers and future `/site` work find the right document quickly.

The docs are organized around the app as it exists today, not around placeholder claims or older planning material.

## Core Product Docs

- [overview.md](/home/kristency/Projects/nodechat/docs/overview.md)  
  Defines what NodeChat is, what it currently does, and what scope it should claim.

- [features.md](/home/kristency/Projects/nodechat/docs/features.md)  
  Describes the main user-facing capabilities of the app in product terms.

- [user-flows.md](/home/kristency/Projects/nodechat/docs/user-flows.md)  
  Explains how a user moves through identity setup, contact management, chatting, group use, and destructive flows.

- [limitations.md](/home/kristency/Projects/nodechat/docs/limitations.md)  
  States the current project boundaries so the app and future site do not overclaim.

## Technical Reference Docs

- [security.md](/home/kristency/Projects/nodechat/docs/security.md)  
  Explains the current security model, including local identity, session establishment, encryption, and trust semantics.

- [message-lifecycle.md](/home/kristency/Projects/nodechat/docs/message-lifecycle.md)  
  Describes how messages move through queued, sent, delivered, and read states.

- [architecture.md](/home/kristency/Projects/nodechat/docs/architecture.md)  
  Gives a concise view of the implemented app layers and runtime flow.

## Project Support Docs

- [contributing.md](/home/kristency/Projects/nodechat/docs/contributing.md)  
  Explains how to contribute in a way that matches the project’s engineering and documentation standards.

- [DOCS_PLAN.md](/home/kristency/Projects/nodechat/docs/DOCS_PLAN.md)  
  Records the documentation strategy and intended structure for future expansion.

## Older Material

The repository also contains older architecture and archive material. Those files may still be useful for background context, but they should not replace the newer app-based docs as the primary source of truth.

Use the current docs first when:

- preparing for project defense
- updating user-facing explanations
- refining `/site` content
- checking what the app should currently claim

## Recommended Reading Order

For project understanding:

1. [overview.md](/home/kristency/Projects/nodechat/docs/overview.md)
2. [features.md](/home/kristency/Projects/nodechat/docs/features.md)
3. [user-flows.md](/home/kristency/Projects/nodechat/docs/user-flows.md)
4. [limitations.md](/home/kristency/Projects/nodechat/docs/limitations.md)
5. [security.md](/home/kristency/Projects/nodechat/docs/security.md)
6. [message-lifecycle.md](/home/kristency/Projects/nodechat/docs/message-lifecycle.md)
7. [architecture.md](/home/kristency/Projects/nodechat/docs/architecture.md)

For `/site` planning:

- start with `overview`, `features`, and `limitations`
- use `user-flows` and `security` for deeper content sections
- use `architecture` and `message-lifecycle` for technical reference pages
