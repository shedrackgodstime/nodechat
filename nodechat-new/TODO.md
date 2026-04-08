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

## ✅ Phase 1: Unified Row & List Experience
*Objective: Eliminate "jampacked" lists and move to a unified component language.*

- [x] **Unified ActionRow**: Replaced `ChatRow` and `ContactRow` with a single `ActionRow` component that supports avatars, badges, and status icons consistently.
- [x] **Centralized Data Models**: Decoupled messaging and network data into `row_data.slint`.
- [x] **Systemic List Highlighting**: Integrated `active-id` selection logic across Desktop, Tablet, and Mobile.
- [x] **Legacy Decommissioned**: Removed outdated row components to ensure a clean, debt-free codebase.

---

## ✅ Phase 2: Interaction & Mobile Excellence
*Objective: Professional mobile behavior (keyboard and focus).*

- [x] **Keyboard-Safe Content**: Shell-level keyboard spacer in `app.slint` protects ALL screens (Settings, Contacts, Chats).
- [x] **Smart Navigation**: Mobile NavBar automatically hides when the keyboard is active to maximize screen space.
- [x] **ScrollView Stabilization**: Locked `viewport-width` on all primary surfaces to prevent horizontal overflow and jitter.

---

## 🛠️ Phase 3: High-Utility Components
*Objective: Standardize forms and modals.*

- [x] **Premium Button System**: Engineered a unified button component (Primary, Secondary, Danger, Ghost) with kinetic animations.
- [ ] **Unified Form Shell**: Create a shared layout for `AddContact`, `ChangePassword`, and `EditName` that places labels and inputs in a predictive, keyboard-aware column.
- [ ] **Premium Modals**: Modernize the `ConfirmModal` backdrop (blur/glassmorphism) and button weighting.

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