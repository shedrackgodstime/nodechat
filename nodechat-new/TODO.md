# NodeChat New — Engineering Backlog

This document tracks the migration progress from the legacy UI to the "nodechat-new" systemic architecture, following the [UI Migration Map](./UI_MIGRATION_MAP.md) and [Slint UI Docs Notes](./SLINT_UI_DOCS_NOTES.md).

---

## ✅ Completed Foundation & Shell
Structural integrity and platform parity are established.

- [x] **Systemic Shell**: Decoupled layout engine with absolute positioning for Sidebar/Content.
- [x] **Responsive Breakpoints**: Explicit `DeviceType` (Mobile/Tablet/Desktop) controllers.
- [x] **Premium Navigation**: 72px NavBar (Desktop/Mobile) and 90px Rail (Tablet).
- [x] **Edge-to-edge Mobile Chrome**: Unified top safe-area with application headers.
- [x] **Platform Parity**: Tablet-specific labels and vertical rail optimized.
- [x] **Zero Loop Logic**: Eliminated `layoutinfo` binding loops in complex shells.
- [x] **Theme System**: Unified `theme.slint` with HSL-balanced colors and 8pt grid tokens.

---

## 🍱 Phase 1: Unified Row & List Experience
*Objective: Eliminate "jampacked" lists and move to a unified component language.*

- [ ] **Unified ActionRow**: Replace `ChatRow` and `ContactRow` with a single `ActionRow` component that supports avatars, badges, and status icons consistently.
- [ ] **List Scaffolding**: Standardize `padding` and `spacing` across all `ChatList`, `ContactsList`, and `Settings` sections.
- [ ] **Empty State Evolution**: Move the "Select a conversation" text into a card-based component that can appear either in the primary area or list area.

---

## ⌨️ Phase 2: Interaction & Mobile Excellence
*Objective: Professional mobile behavior (keyboard and focus).*

- [ ] **Keyboard-Safe Content**: Ensure `VerticalLayout` in the Primary Content area properly respects `root.kb-height` to prevent overlapping with the chat composer.
- [ ] **Global Focus Controller**: Add `clear-focus()` to the `nav-to()` callback in `app.slint` to resolve "ghost focus" on screen switches.
- [ ] **ScrollView Stabilization**: Ensure all primary screens (Chat View, Contacts, Settings) are wrapped in specialized scroll containers that prevent horizontal overflow.

---

## 🛠️ Phase 3: High-Utility Components
*Objective: Standardize forms and modals.*

- [ ] **Unified Form Shell**: Create a shared layout for `AddContact`, `ChangePassword`, and `EditName` that places labels and inputs in a predictive, keyboard-aware column.
- [ ] **Premium Modals**: Modernize the `ConfirmModal` backdrop (blur/glassmorphism) and button weighting.
- [ ] **Input Focus Policy**: Use `forward-focus` correctly on all wrapped input elements to ensure immediate reachability.

---

## ✨ Phase 4: Identity & Onboarding
*Objective: A shorter, faster path to the first chat.*

- [ ] **Splash Screen Port**: Bring over the splash logic to the new shell but with 50% faster transitions.
- [ ] **Identity Hero**: Redesign the Identity Card as a high-contrast card that feels like a physical passport/ID.
- [ ] **PIN/Security Flow**: Update the unlock screen to match the new minimalist layout language.

---

## 🧪 Phase 5: Technical Debt & Polish
*Objective: Clean, maintainable Slint patterns.*

- [ ] **I18n Preparation**: Wrap all user-facing strings in `@tr()` macros.
- [ ] **Accessibility**: Declare `accessibility-role` and `accessibility-label` for all custom interactive items.
- [ ] **Animation Refinement**: Add micro-animations (e.g., subtle height growth or opacity fades) to the `NavItem` indicators.

---

> [!NOTE]
> Priority is strictly: **Phase 1 (Consistency) > Phase 2 (Mobile Stability) > Phase 3 (Forms).**