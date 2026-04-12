# NodeChat New Architecture And Workflow

This document defines how the new NodeChat project should be built so it stays clean, scalable, and easy to explain later.

The target is not only to make the app work.
The target is to make the codebase:

- easy to navigate
- easy to extend
- easy to teach
- easy to present to lecturers
- hard to accidentally bloat
- safe for both mobile and desktop

This is the operating rulebook for the new project.

---

## 1. Main Principle

Build around boundaries, not around screens.

If each layer has one job, the project stays understandable:

- UI renders state and emits actions
- backend coordinates state transitions
- storage handles persistence only
- crypto handles encryption only
- networking handles transport only

The moment a layer starts doing other layers’ work, the project gets packed and difficult to maintain.

---

## 2. Non-Negotiable Rules

### UI rules

- UI never opens SQLite directly
- UI never talks to Iroh directly
- UI never performs crypto directly
- UI never owns business logic that belongs in the backend
- every screen must be built from shared shells and reusable components

### Backend rules

- backend owns command handling
- backend owns app state transitions
- backend owns message delivery status
- backend owns storage queries
- backend owns network coordination
- backend owns crypto state

### Layout rules

- one primary scroll region per page
- no accidental nested scrolling
- mobile and desktop must be designed intentionally
- keyboard and focus behavior must be handled explicitly
- long text must wrap or elide predictably

### Code organization rules

- one concept per file when practical
- repeated structure becomes a component
- repeated state logic becomes a helper
- if a screen starts duplicating structure, move that structure into a shared shell

---

## 3. High-Level Project Shape

The new project should stay split into clear domains.

```text
nodechat-new/
├── src/
│   ├── main.rs
│   ├── app/
│   ├── core/
│   ├── storage/
│   ├── crypto/
│   ├── p2p/
│   └── ui/
├── ui/
│   ├── app.slint
│   ├── components/
│   ├── shells/
│   ├── screens/
│   └── theme.slint
├── assets/
├── notes/
└── docs/
```

You do not need to force this exact tree immediately, but this is the intended direction.

### `src/core/`

Owns the actor loop, commands, events, and state coordination.

### `src/storage/`

Owns schema, migrations, CRUD, and data access helpers.

### `src/crypto/`

Owns identity, key derivation, encryption, and decryption.

### `src/p2p/`

Owns direct transport, gossip transport, discovery, and peer sessions.

### `src/ui/`

Owns callbacks, event-to-UI mapping, and UI bootstrap.

### `ui/`

Owns all Slint markup, design tokens, and visual composition.

---

## 4. Data Flow Contract

The new architecture should move data in one direction at a time.

### UI to backend

The UI sends a `Command` when the user does something:

- send message
- unlock app
- create identity
- add contact
- create group
- change password
- delete conversation
- confirm destructive action

### Backend to UI

The backend emits an `AppEvent` when state changes:

- incoming message
- delivery status update
- contact status update
- identity ready
- unlock success or failure
- conversation state refresh

### Boundary rule

The UI should never infer backend state by reaching into backend internals.
It should react to events and display data passed through explicit models.

---

## 5. Recommended Backend Shape

The backend should stay organized as a worker that owns all core state.

### Suggested core responsibilities

- route commands
- validate input
- update storage
- call crypto helpers
- call network helpers
- emit events back to the UI

### Suggested internal structure

- `commands.rs`
- `worker.rs`
- `events.rs`
- `state.rs`
- `mappers.rs`

The exact filenames can change, but the responsibility split should stay stable.

### Backend design rule

Do not let the worker become a giant switchboard with hundreds of lines of mixed logic.

If a command gets complicated, move the logic into a small private helper with a clear name.

---

## 6. Recommended UI Shape

The UI should be built in layers so it cannot become a pile of duplicated screen markup.

### Root shell

Owns:

- window background
- safe-area padding
- responsive branching
- global overlays
- global modal placement

### Shared shells

Examples:

- `PageShell`
- `ScrollablePageShell`
- `DetailPageShell`
- `ChatShell`
- `OnboardingShell`

These should be the structural backbone of the app.

### Shared components

Examples:

- app header
- conversation header
- message bubble
- row tile
- action row
- card
- confirm dialog
- composer
- status indicator

### Screen components

Screens should only:

- arrange shells
- compose shared components
- pass data in
- emit actions out

Screens should not rewrite global layout rules.

---

## 7. Mobile And Desktop Strategy

The app must feel native on both platforms.

### Mobile

Mobile should prioritize:

- full-width content
- touch-friendly hit targets
- safe-area handling
- bottom-anchored composer
- keyboard-safe scroll behavior
- simplified navigation

### Desktop

Desktop should prioritize:

- persistent navigation
- denser content display
- split-pane chat layouts
- side-by-side detail views where useful
- clearer information hierarchy

### Shared behavior

Both platforms must keep:

- the same state model
- the same command/event contract
- the same data meaning
- the same naming conventions

The presentation changes. The application logic should not.

---

## 8. Keyboard And Focus Policy

This must be handled deliberately from the start.

### Rules

- any editable input must be placed inside a scrollable ancestor when needed
- page switches should clear stale focus
- modal dismissal should clear focus when the modal owned the input
- the chat composer should remain visible when the virtual keyboard appears
- `focus()` and `clear-focus()` should be used intentionally

### Why this matters

Without explicit focus rules, mobile apps become frustrating:

- text cursors appear in the wrong place
- keyboard overlays hide the composer
- focus sticks after navigation
- users think the app is broken even when the backend is fine

This project should not repeat that mistake.

---

## 9. Scroll Policy

Scrolling must be a design decision, not an accident.

### Rules

- one main scroll container per page
- only create nested scroll areas if there is a strong reason
- lists should be delegated to list components
- long text areas should stay within their card or shell
- chat history should be the main scroll area in the chat view

### Chat layout

The chat view should be:

- header
- scrollable message history
- composer

That arrangement keeps the input reachable on mobile and the conversation readable on desktop.

---

## 10. Shared Design System

The UI should use a small design system from the beginning.

### Tokens to define early

- background surfaces
- card surfaces
- text colors
- accent colors
- danger colors
- spacing scale
- border radius scale
- elevation scale
- typography scale

### Why this matters

If tokens are centralized:

- screens stay visually consistent
- new features are easier to add
- the project is easier to re-theme later
- the code is easier to review

---

## 11. Code Style And Readability

The project should read like something a second developer can continue without guessing.

### Naming

- use clear names for modules, functions, and fields
- avoid overly clever abbreviations
- keep command/event names explicit

### Comments

- comment intent, not syntax
- use comments for non-obvious behavior, not for obvious assignments
- document any special cases that affect users or maintainers

### File structure

- keep related code together
- separate public API from private helpers
- avoid giant miscellaneous files
- move reusable layout into shared components instead of copying it

### Review rule

If a file is becoming hard to explain in one paragraph, it probably needs to be split.

---

## 12. Porting Strategy

Do not port the old app by dragging everything over at once.

### Correct order

1. build the new shell
2. build the design tokens
3. build the reusable components
4. build mock screens
5. validate mobile and desktop behavior
6. wire the existing backend contract
7. port one screen at a time
8. delete legacy duplication only after the new path is stable

### Why this matters

This avoids the classic failure mode where the new project becomes old-project-shaped on day one.

---

## 13. Documentation Strategy

The docs are part of the project, not an afterthought.

### Must-have docs

- architecture and workflow
- UI foundation plan
- screen map
- command/event contract
- setup and run instructions
- module responsibilities
- platform notes for desktop and mobile

### Rule for docs

Every important subsystem should have a short description that tells another developer:

- what it does
- what it does not do
- what it depends on
- how to change it safely

---

## 14. What Success Looks Like

The project is in good shape if:

- the UI is visibly consistent
- mobile and desktop both feel intentional
- the backend is easy to reason about
- a new developer can find the right file quickly
- no screen feels overgrown or repetitive
- keyboard and scroll behavior feel deliberate
- the codebase can be explained cleanly to lecturers

That is the standard.

---

## 15. Working Philosophy

The project should be built like a serious final year project, not a rushed prototype.

That means:

- fewer hacks
- clearer boundaries
- more reuse
- better naming
- better documents
- better layout discipline
- better mobile behavior
- better long-term maintainability

If we keep these rules, the app will stay clean instead of getting jammed again.
