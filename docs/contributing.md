# Contributing to NodeChat

## Purpose

This document explains how contributions should be approached in the NodeChat repository.

NodeChat is both:

- a final-year academic project
- a real software codebase with active implementation work

Because of that, contributions should improve the project in a way that is technically sound, professional, and aligned with the current app.

## What Contributions Are Helpful

Useful contributions include:

- bug reports with clear reproduction steps
- fixes for incorrect or inconsistent app behavior
- improvements to code clarity, maintainability, and test coverage
- UI refinements that match the current app direction
- documentation updates based on implemented behavior

The most valuable contributions are the ones that make the app more accurate, more reliable, and easier to defend.

## Contribution Principles

When contributing to NodeChat, follow these rules:

- work from the implemented app, not from assumptions
- avoid placeholder behavior, placeholder text, or unfinished stubs
- keep comments, UI text, and docs professional in tone
- do not overclaim features the app does not fully support
- prefer small, clear, reviewable changes over broad unfocused edits

## Reporting Bugs

When reporting a bug, include:

- what you expected to happen
- what actually happened
- the steps needed to reproduce it
- platform or environment details when relevant

Good bug reports save time and make the project easier to improve.

## Suggesting Features

Feature suggestions are welcome, but they should be grounded in the current project direction.

A strong feature request should explain:

- the problem being solved
- why the feature fits NodeChat
- whether it affects direct chat, group chat, identity, security, storage, or UX

Suggestions are most useful when they extend the existing app clearly rather than changing the project into something unrelated.

## Code Contributions

If you want to contribute code:

1. review the current app behavior first
2. make sure your change matches the project’s existing structure and tone
3. keep the implementation focused
4. verify that the project still builds and tests cleanly after your changes

When changing code, pay attention to:

- Rust and Slint consistency
- message and conversation state correctness
- trust and handshake semantics
- user-facing wording
- destructive-action safety

## Documentation Contributions

Documentation should be written from the app as it currently works.

That means:

- app behavior comes first
- docs follow the implementation
- `/site` content should later grow from the docs, not invent a separate story

If you update documentation, keep it:

- accurate
- concise
- professional
- clear about current scope versus future work

## Pull Request Expectations

A good pull request should:

- explain the problem being addressed
- summarize the change clearly
- note any user-facing impact
- mention verification such as `cargo check` or `cargo test` when relevant

If a change affects app behavior, wording, or trust/security semantics, that should be stated explicitly.

## Standards

Contributors should also follow the project standards already captured in:

- [RULES.md](/home/kristency/Projects/nodechat/docs/RULES.md)
- [AGENT.md](/home/kristency/Projects/nodechat/docs/AGENT.md)

These documents define the engineering expectations for the repository and should not be ignored during implementation work.

## Final Note

NodeChat should continue to read like a serious and well-reasoned project. Every contribution should help strengthen that outcome.
