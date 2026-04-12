# NodeChat — Agent Operational Rules

**Status:** Mandatory for all AI Agent interactions with this codebase.

---

## 1. Core Reference Documents

The agent must always read and follow these documents before generating any code or making any architectural decisions:

| Document | Purpose |
|---|---|
| [ARCHITECTURE.md](./ARCHITECTURE.md) | System design, module hierarchy, Slint/Rust wiring patterns, implementation phases |
| [RULES.md](./RULES.md) | Strict engineering, coding, and testing standards — all code must comply |
| [UX_FLOW.md](./UX_FLOW.md) | Interface design, screen layouts, Slint component specifications, interaction states |

If any instruction conflicts with these documents, flag the conflict explicitly before proceeding.

---

## 2. Code Generation & Implementation

**No guessing how a library works.**
Never assume or infer how a crate's API behaves. If you have not seen the specific version's documentation, say so. The user will provide docs, sample code, or a reference.

**Always verify crate versions.**
Check `Cargo.toml` and ARCHITECTURE.md before writing any code that uses a crate. Use only the API that corresponds to the pinned version. iroh's API changes significantly between versions — do not use examples from a different version. Always check crates.io for the latest stable version and update ARCHITECTURE.md when versions change.

**No placeholder code.**
Every line of code generated must be functional and follow RULES.md. No `todo!()` left in non-scaffold files. No `unwrap()` in non-test code. No placeholder SQL strings. No stub callbacks that do nothing silently.

**If unsure, say so explicitly.**
If you encounter an API, pattern, or concept you cannot verify from the pinned version's documentation, state it clearly: "I am not certain of the correct API for iroh 0.29.0 for this operation — please provide the relevant docs or an example." Do not guess and generate plausible-looking but incorrect code.

---

## 3. Slint-Specific Rules

**Zero business logic in `.slint` files.**
When generating `.slint` files, they contain only: layouts, visual properties, colour token references, animations, and `callback` declarations. Any logic, data fetching, or state management belongs in `src/ui/models.rs` or `src/ui/mod.rs`.

**Colour tokens are always referenced by name.**
Never hardcode hex values in `.slint` files. Use `AppTheme.accent`, `AppTheme.surface-primary`, etc. as defined in `ui/app.slint`.

**All Slint callbacks are wired in `src/ui/mod.rs`.**
When generating callback wiring code, it always goes in `src/ui/mod.rs`, called once during `NodeChatApp::new()`. Never wire callbacks conditionally or inside other callbacks.

**UI updates from the backend always use `invoke_from_event_loop`.**
When generating code that pushes backend state to the UI, always use `slint::invoke_from_event_loop`. Never hold a strong reference to the Slint window from a background thread.

---

## 4. Architecture Compliance

**Before writing any major module, confirm compliance with RULES.md Non-Negotiables.**
Provide a brief statement: "This implementation complies with N-01 (no warnings), N-02 (no unwrap in production code), N-03 (UI thread never blocks)..." — list which rules apply and how the code satisfies them.

**Flag any deviation from ARCHITECTURE.md immediately.**
If a proposed implementation requires a structural change from what ARCHITECTURE.md describes, stop and flag it before writing code. Ask for explicit approval. Do not silently diverge from the architecture.

**The channel boundary is absolute.**
Generated code never passes backend types (iroh `NodeId`, `rusqlite::Connection`, `StaticSecret`, etc.) to the UI layer. Generated code never calls backend functions from Slint callbacks. The only crossing points are `mpsc::Sender<Command>` (UI → backend) and `slint::invoke_from_event_loop` (backend → UI).

---

## 5. Testing Compliance

Any code that implements a testable item from RULES.md section 10.1 (T-01 through T-20) must be accompanied by the corresponding test.

Tests are written in the same commit as the code they test — not deferred.

---

## 6. Communication Style

- If generating code for a module that already has existing code, read the existing code first before generating additions.
- When generating SQL, always use `params![]` — never string formatting.
- When generating crypto code, always include the nonce handling pattern from RULES.md C-02.
- Always note which `.slint` file a UI component belongs to, referencing the screen map in UX_FLOW.md section 6.

---

*Last Updated: 2026-03-30*
*Project: NodeChat — Secure Decentralized Chat*
*Document: Agent Operational Rules v1.2 — Latest Crate Versions*
