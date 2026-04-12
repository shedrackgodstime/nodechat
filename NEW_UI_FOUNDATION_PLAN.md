# NodeChat UI Foundation Plan

## Purpose

This document defines a clean path for rebuilding the NodeChat user interface on a stronger foundation.

The current app already has the hard part working:

- peer-to-peer message flow
- backend command/event architecture
- local storage and app state wiring

What is not clean enough yet is the UI layer:

- too many screens carry their own layout rules
- mobile and desktop behavior is inconsistent
- reusable structure is not centralized enough
- scroll behavior and keyboard behavior are not treated as first-class concerns

The goal is to create a new UI environment with a durable layout system, then port the existing functionality into it step by step.

---

## Core Decision

Build a new UI foundation instead of continuing to pile structure onto the current one.

The new UI should:

- keep the working backend contract
- use a consistent design system
- support mobile and desktop from the same screen model
- keep scrollable content reliable on small screens
- stay easy to maintain as more features are added

This is not a visual tweak. It is a structural reset for the presentation layer.

---

## What Stays

The following parts should remain conceptually stable:

- backend message handling
- command/event flow between UI and worker
- local storage model
- identity and peer data model
- network logic, even if it is reattached later

The UI can be replaced. The backend contract should not be broken unless there is a strong reason.

---

## What Changes

The new UI should replace the current packed layout approach with a proper foundation:

- a root layout shell
- shared page shells
- reusable cards and rows
- consistent spacing and typography
- mobile-first responsive behavior
- desktop-enhanced multi-panel behavior
- scrollable content areas that work on phones
- predictable keyboard and focus handling

---

## Design Principles

### 1. One system, two presentations

Mobile and desktop should not feel like two different apps.

They should share:

- the same information hierarchy
- the same component language
- the same state model

But they can differ in presentation:

- mobile: stacked, full-width, touch-first
- desktop: split-panel, denser, persistent sidebar

### 2. Content first

Screens should be built around the user task, not around the frame.

Examples:

- chat list should feel like a list of conversations, not a dashboard
- chat view should feel like one focused thread
- settings and details pages should behave like clear information cards

### 3. Scroll is a feature

Scrollable areas must be intentional.

Rules:

- a page should not create accidental nested scrolling
- chat message lists should scroll independently
- long cards or forms must remain usable on mobile
- keyboard presence must not trap important content off-screen

### 4. Reuse before duplication

If a header, row, button, card, or shell appears more than once, it should become a shared component.

### 5. State should stay boring

UI components should receive data, render it, and emit actions.

They should not own business logic that belongs in the backend.

---

## Proposed New UI Structure

The new UI should be organized around layers.

### Root shell

The root shell should own:

- app background
- safe-area handling
- responsive breakpoint logic
- top-level navigation frame
- onboarding and lock overlays
- global modals

### Shared shells

Use small wrappers for the repeated page types:

- `PageShell`
- `ScrollablePageShell`
- `DetailPageShell`
- `ChatShell`

These shells should control:

- margins
- maximum content width
- vertical spacing
- title and subtitle placement
- back button placement

### Shared components

Reusable components should include:

- app header
- conversation header
- action row
- contact row
- message bubble
- grouped invite bubble
- input composer
- confirm modal
- status indicator
- empty state block

### Screen layer

Screens should be thin. A screen should mostly:

- declare the page structure
- place shared components
- bind to state
- wire callbacks

The screen itself should not duplicate layout rules that belong to the shell.

---

## Responsive Layout Rules

### Mobile

Mobile should use:

- full-width content
- small outer gutters
- vertical stacking
- large touch targets
- one primary scroll region per page

Mobile should avoid:

- centered layouts that waste space
- hidden important actions
- tiny tap zones
- unnecessary side panels

### Desktop

Desktop should use:

- persistent side navigation where appropriate
- split-pane conversation layouts
- left-aligned content
- wider cards and detail panels
- stronger density without clutter

Desktop should avoid:

- overly large centered blocks
- mobile-style padding copied directly into wide screens
- duplicated navigation chrome

### Shared rules

- content must never overflow horizontally
- text should wrap or ellipsize predictably
- cards must respect parent width
- page bodies should remain visually aligned
- the same screen should feel like the same product across platforms

---

## Layout Model

### Main app

The main app should be built around a simple structure:

- sidebar or navigation area
- active content area
- optional detail panel
- overlays for onboarding, locking, and confirmation

### Chat view

Chat view should be treated as its own layout type:

- header at the top
- message list in the middle
- composer at the bottom

Rules:

- the message list should be the main scrollable region
- the composer should remain accessible
- new messages should not yank the user away from older content if they are reading history
- on mobile, the keyboard must not cover the composer or the latest messages

### Detail pages

Contacts, settings, contact details, group creation, and actions pages should use a consistent detail-page shell.

This shell should provide:

- safe area padding
- max content width
- header block
- section grouping
- scrollable body

---

## Visual Direction

The visual system should be clean, calm, and consistent.

### Typography

- one primary text scale
- clear hierarchy for title, subtitle, body, label, and metadata
- no random font decisions per screen

### Surfaces

- use a small number of surface levels
- background, panel, card, inset, and overlay should be distinct
- keep elevation subtle

### Color behavior

- use color to indicate state, not decoration
- online, relay, verified, warning, and error states should be visually consistent
- outgoing and incoming messages should be easy to distinguish

### Spacing

- spacing should come from a small token set
- avoid one-off spacing values unless there is a good reason

### Shape

- cards should share one radius language
- buttons and list rows should feel related

### Motion

- motion should be minimal and purposeful
- use transitions to clarify state changes, not to decorate every interaction

---

## Screen Strategy

The current UI should be broken into a clear screen map.

### Priority screens

1. Welcome and onboarding
2. Identity setup
3. Identity card / launch
4. Lock screen
5. Chat list
6. Chat view
7. Contacts
8. Contact details
9. Add contact
10. Group creation
11. Select members
12. Conversation actions
13. Settings
14. Edit display name
15. Diagnostics

### Screen composition rule

Each screen should be built from:

- one shell
- one or more shared components
- minimal page-specific logic

If a screen is visually similar to another screen, it should share structure instead of being rewritten from scratch.

---

## Component Foundation

The new UI should standardize the following building blocks.

### 1. App header

Use for:

- page title
- back button
- optional action button
- optional subtitle

### 2. Conversation header

Use for:

- direct chat
- group chat
- contact detail context

It should support:

- title
- initials or avatar
- online status
- relay status
- verified status
- session readiness

### 3. Action row

Use for:

- open chat
- clear history
- delete contact
- exit group
- retry queue
- verification actions

### 4. Message bubble

Use for:

- direct messages
- group messages
- invite messages
- status messages

### 5. Confirm modal

Use for:

- destructive actions
- PIN verification
- group and identity deletion

### 6. Composer

Use for:

- message input
- send button
- attach actions if added later
- keyboard-safe behavior

### 7. Empty state

Use for:

- no chats
- no messages
- no contacts
- no selected conversation

---

## Scroll Policy

Scrolling must be handled carefully from the start.

### Rules

- each page should have at most one primary scroll container
- nested scroll should be used only where it is clearly needed
- cards should expand naturally rather than forcing overflow
- long conversation history should remain navigable on mobile

### Chat scrolling

Chat view must support:

- stable bottom anchoring when the user is near the bottom
- manual reading without forced jumps
- keyboard-safe composer positioning

### Details scrolling

Detail pages should use:

- a single scrollable body
- section spacing that stays readable on small screens
- content width limits on desktop

---

## Focus and Keyboard Handling

Focus handling should be treated as a first-class issue.

Known behavior to avoid:

- cursor still appearing after leaving a screen
- input focus sticking when the page changes
- keyboard covering key controls on mobile

The new UI should define:

- how focus is cleared on page transitions
- how inputs are dismissed when leaving a page
- how scroll adjusts when the keyboard appears
- how the chat composer stays usable on touch devices

This should be verified early, not patched late.

---

## Data and State Boundary

Keep the UI and backend separated by a stable contract.

### UI receives

- chat previews
- message lists
- contact rows
- group member selection data
- connection and identity state
- error and status updates

### UI sends

- create identity
- unlock
- send message
- create group
- add contact
- verify contact
- change password
- destructive actions

### Boundary rule

The UI should never talk directly to storage or networking.

It should only:

- render state
- dispatch commands
- react to events

---

## Suggested Migration Plan

### Phase 1: Freeze and define

- stop adding new UI features to the current layout
- document the new component system
- define screen shells and layout rules
- keep backend contracts stable

### Phase 2: New UI shell

- build the new app shell
- add responsive width rules
- add shared background and safe-area handling
- add top-level overlays and modal support

### Phase 3: Shared components

- rebuild headers
- rebuild rows
- rebuild cards
- rebuild message bubbles
- rebuild composer and confirm modal

### Phase 4: Mock screens

- create screens with mock data first
- validate mobile and desktop behavior
- validate scrolling and keyboard behavior
- validate visual consistency

### Phase 5: Backend port

- connect the existing command/event flow
- map real app state into the new UI
- migrate screen by screen
- keep the old and new states compatible until the switch is complete

### Phase 6: Cleanup

- remove duplicated legacy UI pieces
- remove unused screen branches
- tighten spacing and type scale
- verify the app on desktop and mobile

---

## Porting Order

The safest order for porting is:

1. root shell
2. shared design tokens
3. chat list
4. conversation view
5. contacts and contact details
6. add contact and group flows
7. settings
8. diagnostics and destructive actions
9. onboarding and lock flow polish

This order keeps the working conversation flow intact while the surrounding structure gets replaced.

---

## Foundation Rules

These rules should be treated as constraints for the new UI project.

- do not duplicate the same header structure in multiple files
- do not hardcode arbitrary spacing values everywhere
- do not let screen files become layout dumping grounds
- do not use desktop-only assumptions in shared components
- do not assume keyboard-less behavior on mobile
- do not allow content to overflow its container
- do not split the same concept into separate components unless there is a real reason
- do not let backend code leak into presentation logic

---

## Success Criteria

The new UI foundation is successful if:

- mobile and desktop both feel intentional
- the app is easy to navigate
- scroll behavior is reliable
- components are reused instead of copied
- the chat flow remains clear
- the project is easier to extend without breaking consistency
- the interface looks like one product, not a collection of screens

---

## Immediate Next Steps

1. confirm the new project boundary
2. define the root shell and screen shells
3. define the shared design tokens
4. choose the component set to build first
5. wire mock data before live backend data
6. port the working message flow into the new layout

---

## Final Note

The best way forward is not to keep adding pieces to the current packed UI.
The best way forward is to build a proper UI base, then move the working backend into it.

That gives NodeChat a long-term path that is cleaner, more portable, and easier to scale across mobile and desktop.
