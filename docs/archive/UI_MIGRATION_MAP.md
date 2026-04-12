# NodeChat UI Migration Map

This document translates the old Slint UI into a cleaner `nodechat-new` layout strategy.
The goal is to keep the useful visual patterns, redesign the structural debt, and drop the parts that make the old project feel packed.

## 1. Core Decisions

### Keep

- Dark theme direction and high-contrast accent palette.
- Card-based rows, panels, and sections.
- Clear separation between list, detail, and settings surfaces.
- Safe-area and keyboard-aware layout rules.
- Touch-friendly controls with large hit areas.

### Redesign

- Root navigation should become a small shell, not a giant screen router.
- Shared headers should be unified into one shell system.
- Mobile and desktop should share components but not the same layout assumptions.
- Forms should use a single scrollable content surface and one input-focus policy.

### Drop

- Deeply nested conditional screen trees in the root app.
- Repeated header logic across multiple screens.
- One-off layout hacks that only solve a single page.
- Hardcoded spacing and sizing that cannot scale across devices.

**Reference:** [`ui/app.slint`](/home/kristency/Projects/nodechat/ui/app.slint), [`ui/theme.slint`](/home/kristency/Projects/nodechat/ui/theme.slint), [`ui/components/base_screen.slint`](/home/kristency/Projects/nodechat/ui/components/base_screen.slint)

## 2. App Shell

### Keep

- Desktop split-view idea.
- Mobile single-screen flow.
- Clear app title and branded top-level identity.

### Redesign

- Move routing out of a giant root file and into a thin UI state controller.
- Use one app shell for chrome, navigation, and responsive breakpoints.
- Make the shell own safe areas, keyboard space, and primary panel selection.

### Drop

- The current root file doing routing, state mutation, overlays, and per-page wiring all at once.

**Reference:** [`ui/app.slint`](/home/kristency/Projects/nodechat/ui/app.slint), [`ui/screens/chat_list.slint`](/home/kristency/Projects/nodechat/ui/screens/chat_list.slint), [`ui/screens/chat_view.slint`](/home/kristency/Projects/nodechat/ui/screens/chat_view.slint)

## 3. Chat List

### Keep

- Chat rows with avatar, timestamp, unread count, and status badges.
- Empty state with a helpful next action.
- Desktop sidebar feel.

### Redesign

- Use one reusable list card component for chats, contacts, and group entries.
- Keep list scrolling inside a single surface.
- Move bottom navigation into a cleaner shell section or a desktop-only navigation rail.
- Simplify the empty state into a clearer onboarding card.

### Drop

- Duplicate chat row presentation that mixes list content with unrelated navigation chrome.
- Hardwired bottom navigation inside the list view for every mode.

**Reference:** [`ui/screens/chat_list.slint`](/home/kristency/Projects/nodechat/ui/screens/chat_list.slint), [`ui/components/contact_row.slint`](/home/kristency/Projects/nodechat/ui/components/contact_row.slint)

## 4. Chat View

### Keep

- Message bubbles with mine vs incoming styling.
- Invite bubble support.
- Status and verification cues.
- Composer anchored at the bottom.

### Redesign

- Make the message area and composer feel like one coherent chat surface.
- Keep the composer keyboard-safe on mobile.
- Use one focus policy so the input stays visible when the keyboard opens.
- Separate the header, banner, message stream, and composer into stable layout zones.

### Drop

- Nested layout hacks that try to solve scrolling and keyboard behavior in many places.
- Duplicated chat header logic outside the chat shell.

**Reference:** [`ui/screens/chat_view.slint`](/home/kristency/Projects/nodechat/ui/screens/chat_view.slint), [`ui/components/message_bubble.slint`](/home/kristency/Projects/nodechat/ui/components/message_bubble.slint), [`ui/components/chat_input.slint`](/home/kristency/Projects/nodechat/ui/components/chat_input.slint), [`ui/components/group_invite_bubble.slint`](/home/kristency/Projects/nodechat/ui/components/group_invite_bubble.slint)

## 5. Contacts

### Keep

- Quick action card pattern.
- Directory list pattern.
- Clear separation between peer actions and peer entries.

### Redesign

- Make the quick actions feel like a compact top section instead of a large secondary page.
- Use the same row component language as chats.
- Keep contact details in a dedicated detail view, not a dense list row.

### Drop

- Overly verbose card stacking when the action could be one simple button row.

**Reference:** [`ui/screens/contacts.slint`](/home/kristency/Projects/nodechat/ui/screens/contacts.slint), [`ui/components/contact_row.slint`](/home/kristency/Projects/nodechat/ui/components/contact_row.slint)

## 6. Contact Details And Conversation Actions

### Keep

- Identity hero card.
- Trust/verification state.
- Copyable technical identifiers.
- Destructive actions separated from normal actions.

### Redesign

- Merge related detail pages into one clearer peer-info surface.
- Keep destructive actions in a consistent danger section.
- Use progressive disclosure so basic peer info appears first, technical data second, actions last.

### Drop

- Duplicate “peer info” pages that split the same data across too many screens.

**Reference:** [`ui/screens/contact_details.slint`](/home/kristency/Projects/nodechat/ui/screens/contact_details.slint), [`ui/screens/conversation_actions.slint`](/home/kristency/Projects/nodechat/ui/screens/conversation_actions.slint)

## 7. Settings

### Keep

- Profile card.
- Grouped settings sections.
- Clear danger zone separation.

### Redesign

- Convert settings into a predictable list of sections with one action type per row.
- Move technical actions into a debug or advanced area.
- Keep copy-to-clipboard flows visually obvious.

### Drop

- Multiple special-case cards that each behave differently without a shared row system.

**Reference:** [`ui/screens/settings.slint`](/home/kristency/Projects/nodechat/ui/screens/settings.slint), [`ui/screens/change_password.slint`](/home/kristency/Projects/nodechat/ui/screens/change_password.slint), [`ui/screens/diagnostics.slint`](/home/kristency/Projects/nodechat/ui/screens/diagnostics.slint)

## 8. Forms And Input Screens

### Keep

- Single-purpose input screens.
- Clear labels and helper text.
- Large CTA buttons.

### Redesign

- Build all input screens around the same form shell.
- Keep inputs visible and reachable with keyboard open.
- Use one validation pattern for all text fields.

### Drop

- Repeated bespoke input layouts that differ only slightly.

**Reference:** [`ui/screens/add_contact.slint`](/home/kristency/Projects/nodechat/ui/screens/add_contact.slint), [`ui/screens/create_group.slint`](/home/kristency/Projects/nodechat/ui/screens/create_group.slint), [`ui/screens/edit_name.slint`](/home/kristency/Projects/nodechat/ui/screens/edit_name.slint), [`ui/screens/change_password.slint`](/home/kristency/Projects/nodechat/ui/screens/select_members.slint)

## 9. Onboarding And Identity

### Keep

- Friendly first-run flow.
- Identity card reveal.
- Clear launch-to-chats ending.

### Redesign

- Make the onboarding path shorter and easier to understand.
- Avoid loading too many visual concepts at once.
- Treat identity generation as a clean step sequence.

### Drop

- Over-styled intro screens that delay the first meaningful interaction.

**Reference:** [`ui/screens/splash.slint`](/home/kristency/Projects/nodechat/ui/screens/splash.slint), [`ui/screens/welcome.slint`](/home/kristency/Projects/nodechat/ui/screens/welcome.slint), [`ui/screens/identity_card.slint`](/home/kristency/Projects/nodechat/ui/screens/identity_card.slint)

## 10. Shared Components

### Keep

- Reusable chat rows.
- Reusable bubbles.
- Reusable confirmation modal concept.

### Redesign

- Make the component library smaller but more consistent.
- Standardize row heights, padding, and icon treatment.
- Split visual components from behavior-heavy components where needed.

### Drop

- Components that only exist to support one page and do not generalize.

**Reference:** [`ui/components/contact_row.slint`](/home/kristency/Projects/nodechat/ui/components/contact_row.slint), [`ui/components/message_bubble.slint`](/home/kristency/Projects/nodechat/ui/components/message_bubble.slint), [`ui/components/confirm_modal.slint`](/home/kristency/Projects/nodechat/ui/components/confirm_modal.slint), [`ui/components/action_row.slint`](/home/kristency/Projects/nodechat/ui/components/action_row.slint)

## 11. What The New Project Should Be

The new UI should behave like a product shell:

- one design system
- one routing model
- one set of shared form rules
- one chat surface model
- one responsive strategy for mobile and desktop

That is the difference between a project that is easy to present and a project that becomes hard to explain later.

**Reference:** [`nodechat-new/ARCHITECTURE_AND_WORKFLOW.md`](/home/kristency/Projects/nodechat/nodechat-new/ARCHITECTURE_AND_WORKFLOW.md), [`nodechat-new/NEW_UI_FOUNDATION_PLAN.md`](/home/kristency/Projects/nodechat/nodechat-new/NEW_UI_FOUNDATION_PLAN.md), [`nodechat-new/SLINT_UI_DOCS_NOTES.md`](/home/kristency/Projects/nodechat/nodechat-new/SLINT_UI_DOCS_NOTES.md)

