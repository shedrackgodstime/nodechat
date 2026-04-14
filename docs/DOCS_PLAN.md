# NodeChat Documentation Plan

## Purpose

This plan defines how NodeChat documentation should be structured going forward.

The documentation must serve two audiences at the same time:

1. project defense and academic review
2. future public-facing site content in `/site`

Because of that, the docs must be:

- accurate to the current app
- professional in tone
- clear about current behavior versus future work
- modular enough to be reused later by the website

The docs should not be written from assumptions, placeholder copy, or older architecture ideas. They should be written from the implemented app and its current behavior.

---

## Current Observations

### 1. The app is now ahead of the docs

The app code, wording, and behavior have already been tightened in several areas:

- peer verification is now separate from handshake/session readiness
- status labels are more accurate
- destructive flows are more consistent
- in-app feedback is visible through global notices
- the About page is now an app-facing summary, not a placeholder mini-site

The docs should be updated to match this reality.

### 2. Existing docs are too architecture-heavy as a starting point

The current docs tree contains useful material, but much of it is written as:

- implementation theory
- broad architecture narrative
- planning/history content
- placeholder assumptions

That material is still useful, but it should not be the primary basis for the next documentation pass.

### 3. The `/site` placeholder shows the intended public content model

The site is not final, but it clearly indicates the intended direction:

- a clear landing page
- concise feature explanations
- a documentation index
- project/team context
- contribution/support paths

This means the docs should be written in sections that can later power:

- feature summaries
- landing page content
- docs cards and routes
- FAQ/support sections
- architecture summaries

### 4. The About screen should not be the source of truth

The About page now has the correct role:

- short app summary
- project context
- essential links
- diagnostics access

It should remain compact.

The full story belongs in docs.

---

## Documentation Principles

These rules should guide every new or rewritten document.

### Rule 1: Write from the implemented app

If the app does not clearly do it yet, the docs must not present it as finished behavior.

### Rule 2: Separate current behavior from future work

Every limitation or planned improvement should be explicitly marked as:

- current limitation
- future work
- out of scope

### Rule 3: Define terms consistently

The same terms should mean the same thing everywhere.

Important terms that must stay consistent:

- identity
- connection ticket
- contact
- direct conversation
- group conversation
- handshake
- secure session
- verified contact
- queued
- sent
- delivered
- read

### Rule 4: Prefer layered documentation over one giant document

Short focused docs are easier to defend, maintain, and later reuse in `/site`.

### Rule 5: Product-first, then architecture

The documentation order should begin with:

- what NodeChat is
- what it does
- how a user moves through it

Then move into:

- system design
- storage
- crypto
- transport
- limitations

---

## Recommended Documentation Structure

### Layer 1: Core Product Docs

These are the most important docs because they support both defense and `/site`.

#### `docs/overview.md`

Purpose:

- define what NodeChat is
- define project scope
- summarize supported capabilities
- summarize current maturity level

This should become the primary source for the site homepage story.

#### `docs/features.md`

Purpose:

- describe major app capabilities in product terms
- explain direct chat, group chat, local identity, local storage, delivery states

This should become the primary source for site feature cards and feature sections.

#### `docs/user-flows.md`

Purpose:

- explain how the app is used from start to finish

Suggested sections:

- create identity
- unlock app
- add contact
- start direct chat
- create group
- invite contacts
- accept group invite
- clear history
- reset identity

#### `docs/limitations.md`

Purpose:

- clearly state what the app does not yet fully solve
- prevent overclaiming during defense or on the future site

Examples:

- relay visibility is limited
- advanced trust verification workflow may still be basic
- offline delivery scope should be stated carefully
- group key management limits should be documented honestly

### Layer 2: Technical Reference Docs

These docs should explain how the app works internally, based on current implementation.

#### `docs/architecture.md`

Purpose:

- explain the major system layers
- show how UI, backend, storage, crypto, and transport connect

This should be shorter and more implementation-aligned than the current large architecture document.

#### `docs/security.md`

Purpose:

- explain what is encrypted
- explain how session establishment works
- explain what verification means
- explain what is stored locally

This is especially important now that handshake and verification have been logically separated.

#### `docs/message-lifecycle.md`

Purpose:

- explain the meaning of message states
- document the visible lifecycle:
  - queued
  - sent
  - delivered
  - read

This should match the actual backend and UI behavior exactly.

#### `docs/data-models.md`

Purpose:

- define the main stored entities:
  - local identity
  - peer/contact
  - group
  - message

### Layer 3: Project and Support Docs

These support maintainability and future site/docs navigation.

#### `docs/setup.md`

Purpose:

- project prerequisites
- environment setup
- desktop build/run steps

#### `docs/testing.md`

Purpose:

- how to run checks and tests
- what currently exists in automated coverage

#### `docs/contributing.md`

Purpose:

- contribution expectations
- code quality expectations
- issue/PR etiquette

#### `docs/faq.md`

Purpose:

- answer common defense questions
- answer likely user questions later for the site

Examples:

- Is NodeChat serverless?
- What does verified mean?
- What happens if a peer is offline?
- Where are messages stored?
- Does the app require registration?

---

## Recommended Writing Order

The order below is designed to create a stable foundation for both defense and the future site.

### Phase 1: Foundation

1. `docs/overview.md`
2. `docs/features.md`
3. `docs/user-flows.md`
4. `docs/limitations.md`

### Phase 2: Technical Clarity

5. `docs/security.md`
6. `docs/message-lifecycle.md`
7. `docs/architecture.md`
8. `docs/data-models.md`

### Phase 3: Project Support

9. `docs/setup.md`
10. `docs/testing.md`
11. `docs/faq.md`
12. update `docs/contributing.md` if needed

---

## How This Supports `/site`

The future site can map directly to these documents.

### Landing Page

Primary source docs:

- `overview.md`
- `features.md`
- `limitations.md` for careful claim boundaries

### Docs Index

Primary source docs:

- all markdown files in `docs/`

### Support / Community / Contribution

Primary source docs:

- `contributing.md`
- `faq.md`
- issue tracker links

### Product Story Pages

Primary source docs:

- `user-flows.md`
- `security.md`
- `architecture.md`

The site should later summarize and present these docs, not invent parallel truth.

---

## What Should Change In Existing Docs

The current docs should be reviewed with these actions in mind:

- keep useful reference material
- archive outdated assumptions
- reduce duplicated explanations
- rewrite broad placeholder claims to match the app
- avoid leading with architecture when product behavior should come first

The current large architecture and UX documents can still remain as detailed references, but they should not be the only or primary entry point anymore.

---

## Immediate Next Step

Start with:

1. `docs/overview.md`

Why:

- it anchors the product story
- it gives the future site a reliable summary source
- it sets the vocabulary for the rest of the docs
- it forces all later docs to align with the actual app instead of older assumptions

Once `overview.md` is approved, continue with:

2. `docs/features.md`
3. `docs/user-flows.md`

That creates the strongest base before rewriting deeper architecture material.
