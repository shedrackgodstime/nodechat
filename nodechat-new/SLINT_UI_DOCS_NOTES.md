# Slint UI Docs Notes for NodeChat

This file collects the Slint documentation patterns that are directly useful for the NodeChat UI rebuild.

The goal is not to copy Slint examples verbatim. The goal is to extract the rules that will help NodeChat become:

- cleaner
- more consistent
- responsive on mobile and desktop
- easier to maintain
- easier to scale with new screens

---

## 1. Start With Layout, Not Pixels

The Slint layout docs make one point very clearly: layout elements are the scalable way to build non-trivial UIs. Explicit `x`, `y`, `width`, and `height` can work for simple scenes, but complex apps should be built out of layouts, size constraints, and nested containers.

What this means for NodeChat:

- use shared layout shells instead of screen-specific positioning
- rely on `HorizontalLayout`, `VerticalLayout`, and `GridLayout` for structure
- use `min-width`, `min-height`, `preferred-width`, and stretch properties to control behavior
- keep one screen from “fighting” another by inventing its own sizing rules

This is especially important for:

- mobile chat layouts
- desktop split panels
- detail pages
- forms and settings

Reference:

- [`materials/slint-ui_docs/guide/language/coding/positioning-and-layouts.mdx`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/guide/language/coding/positioning-and-layouts.mdx)
- [`materials/slint-ui_docs/reference/layouts/overview.mdx`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/reference/layouts/overview.mdx)
- [`materials/slint-ui_docs/reference/std-widgets/globals/stylemetrics.mdx`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/reference/std-widgets/globals/stylemetrics.mdx)

---

## 2. Treat Repetition as a Model Problem

The docs encourage using `for ... in` with arrays and models instead of hand-building repeated UI nodes. That matters because most NodeChat screens are repeated-item screens:

- chat previews
- contacts
- group members
- actions
- messages

The important lesson is not just “use loops.” It is:

- keep repeated rows data-driven
- define a proper model shape
- let the UI render from that model
- keep selection state explicit

This is a strong fit for NodeChat because it reduces manual duplication and makes the UI easier to port screen by screen.

Reference:

- [`materials/slint-ui_docs/guide/language/coding/repetition-and-data-models.mdx`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/guide/language/coding/repetition-and-data-models.mdx)
- [`materials/slint-ui_docs/reference/std-widgets/views/listview.mdx`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/reference/std-widgets/views/listview.mdx)
- [`materials/slint-ui_docs/reference/std-widgets/views/standardlistview.mdx`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/reference/std-widgets/views/standardlistview.mdx)

---

## 3. Make Focus Explicit

The focus docs are directly relevant to the problem you already observed in NodeChat: inputs can keep focus visually even after leaving a page, especially on mobile.

Useful rules from the docs:

- focus must be intentionally assigned with `focus()`
- focus must be intentionally removed with `clear-focus()`
- wrapped components should use `forward-focus`
- `Window` can use `forward-focus` to define the initial focus target

For NodeChat, this means:

- input components should not rely on accidental focus state
- page switches should clear focus from obsolete inputs
- chat composer focus should be controlled when entering and leaving a conversation
- onboarding and modal flows should actively manage focus instead of hoping the framework does it automatically

This is one of the most important docs for the new UI base.

Reference:

- [`materials/slint-ui_docs/guide/development/focus.mdx`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/guide/development/focus.mdx)
- [`materials/slint-ui_docs/reference/keyboard-input/focusscope.mdx`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/reference/keyboard-input/focusscope.mdx)
- [`materials/slint-ui_docs/reference/keyboard-input/textinput.mdx`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/reference/keyboard-input/textinput.mdx)

---

## 4. Use Custom Controls for Reuse

The custom controls guide shows the right mental model for a scalable Slint codebase:

- build reusable components
- expose clean properties
- bind callbacks explicitly
- keep the root component thin

This is exactly what NodeChat needs if the UI is going to be rebuilt cleanly.

Useful lessons:

- a component can expose only the properties it needs
- callbacks are the clean way to report user actions
- property bindings keep state propagation simple
- custom controls can wrap lower-level widgets without leaking implementation details

For NodeChat, this supports a component library like:

- `AppHeader`
- `ConversationHeader`
- `PageShell`
- `ActionRow`
- `MessageBubble`
- `ConfirmModal`
- `Composer`

Reference:

- [`materials/slint-ui_docs/guide/development/custom-controls.mdx`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/guide/development/custom-controls.mdx)
- [`materials/slint-ui_docs/reference/std-widgets/overview.mdx`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/reference/std-widgets/overview.mdx)
- [`materials/slint-ui_docs/reference/window/window.mdx`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/reference/window/window.mdx)

---

## 5. Scrollable Regions Should Be Intentional

The ScrollView and ListView docs matter because NodeChat will depend heavily on scrolling:

- chat history
- chat list
- contacts
- group member selection
- settings pages

The key takeaways:

- `ScrollView` has explicit viewport sizes
- `ListView` is for repeated content and only instantiates visible elements
- `StandardListView` gives a ready-made list model when you do not need a custom delegate
- horizontal overflow should be treated as a bug, not as normal behavior

For NodeChat, this suggests:

- use a primary scroll region per page
- use list views for repeatable collections
- make message history its own scrollable area
- keep the composer separate from the message list

Reference:

- [`materials/slint-ui_docs/reference/std-widgets/views/scrollview.mdx`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/reference/std-widgets/views/scrollview.mdx)
- [`materials/slint-ui_docs/reference/std-widgets/views/listview.mdx`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/reference/std-widgets/views/listview.mdx)
- [`materials/slint-ui_docs/reference/std-widgets/views/standardlistview.mdx`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/reference/std-widgets/views/standardlistview.mdx)
- [`materials/slint-ui_docs/material-components/ui/components/scroll_view.slint`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/material-components/ui/components/scroll_view.slint)

---

## 6. Material App Bars and Navigation Are a Good Shell Pattern

The Material component docs are very useful for deciding how NodeChat should present navigation on different form factors.

What the docs show:

- `AppBar` for top-level headers and actions
- `NavigationDrawer` for grouped side navigation
- `NavigationRail` for desktop or wide layouts
- `NavigationBar` for mobile bottom navigation

For NodeChat, this maps well to the idea of:

- desktop: persistent left navigation or split panel
- mobile: stacked navigation and a simpler primary shell
- shared screen content underneath both presentations

The important lesson is that navigation should be a shell concern, not duplicated across every screen.

Reference:

- [`materials/slint-ui_docs/material-components/docs/src/content/docs/components/AppBars/app_bar.mdx`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/material-components/docs/src/content/docs/components/AppBars/app_bar.mdx)
- [`materials/slint-ui_docs/material-components/docs/src/content/docs/components/Navigation/navigation_drawer.mdx`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/material-components/docs/src/content/docs/components/Navigation/navigation_drawer.mdx)
- [`materials/slint-ui_docs/material-components/docs/src/content/docs/components/Navigation/navigation_rail.mdx`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/material-components/docs/src/content/docs/components/Navigation/navigation_rail.mdx)
- [`materials/slint-ui_docs/material-components/docs/src/content/docs/components/AppBars/navigation_bar.mdx`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/material-components/docs/src/content/docs/components/AppBars/navigation_bar.mdx)

---

## 7. Cards and List Tiles Should Carry Most of the Visual Weight

The Material card and list tile docs are directly relevant to NodeChat’s core screens.

Useful takeaways:

- cards are meant to group related content and actions
- clickable cards should behave like interactive surfaces
- list tiles are ideal for single-row items with optional supporting text and avatar areas

For NodeChat, this is the right direction for:

- chat previews
- contact rows
- member rows
- settings entries
- action rows
- invite rows

This also helps establish a cleaner visual language across the app. If a row is meant to be a row, it should look and behave like one. If a piece of content is important enough to deserve emphasis, it should become a card.

Reference:

- [`materials/slint-ui_docs/material-components/ui/components/card.slint`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/material-components/ui/components/card.slint)
- [`materials/slint-ui_docs/material-components/docs/src/content/docs/components/Cards/filled_card.mdx`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/material-components/docs/src/content/docs/components/Cards/filled_card.mdx)
- [`materials/slint-ui_docs/material-components/docs/src/content/docs/components/list_tile.mdx`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/material-components/docs/src/content/docs/components/list_tile.mdx)
- [`materials/slint-ui_docs/material-components/ui/components/list.slint`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/material-components/ui/components/list.slint)

---

## 8. Text Fields, Inputs, and Validation Need Clear States

The text field docs are useful because they expose exactly the state NodeChat needs for onboarding, unlock, password changes, and search/add-contact flows:

- label
- placeholder text
- supporting text
- error state
- focus state
- selection helpers

This supports a more disciplined approach to forms.

For NodeChat, the main lesson is:

- inputs should communicate state clearly
- errors should be visible and local to the field
- focus and validation should be tied together
- password and PIN flows should not be ad hoc

That is especially relevant for:

- unlock screen
- create identity screen
- change password screen
- add contact flow

Reference:

- [`materials/slint-ui_docs/material-components/docs/src/content/docs/components/text_field.mdx`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/material-components/docs/src/content/docs/components/text_field.mdx)
- [`materials/slint-ui_docs/reference/std-widgets/views/lineedit.mdx`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/reference/std-widgets/views/lineedit.mdx)
- [`materials/slint-ui_docs/reference/std-widgets/views/textedit.mdx`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/reference/std-widgets/views/textedit.mdx)

---

## 9. Modals and Dialogs Should Be Reserved for Important Actions

The dialog docs confirm that modal overlays are appropriate for confirmations, destructive actions, and any flow that needs deliberate user acknowledgment.

What to copy:

- use dialogs for important decisions
- keep the default action and secondary actions explicit
- do not hide destructive actions inside normal page content

For NodeChat, this is a good fit for:

- delete identity
- clear messages
- delete conversation
- clear conversation history
- PIN verification steps

This also matches your current app’s need for safe confirmation flows while keeping the underlying state transitions predictable.

Reference:

- [`materials/slint-ui_docs/material-components/docs/src/content/docs/components/Dialogs/dialog.mdx`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/material-components/docs/src/content/docs/components/Dialogs/dialog.mdx)
- [`materials/slint-ui_docs/material-components/ui/components/modal.slint`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/material-components/ui/components/modal.slint)
- [`materials/slint-ui_docs/material-components/ui/components/bottom_sheet.slint`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/material-components/ui/components/bottom_sheet.slint)

---

## 10. Accessibility and Translation Should Be Planned Early

The best-practices docs make two points that should shape the new NodeChat UI from the start:

- accessibility properties should be declared early for custom components
- translatable text should be wrapped in `@tr(...)`

This matters because the new UI is a good chance to stop baking in hard-to-change assumptions.

For NodeChat:

- custom buttons, cards, list items, and dialogs should expose sensible accessibility roles
- user-visible strings should not be concatenated casually
- component text should be structured so translation is possible later

This is a structural quality issue, not a cosmetic one.

Reference:

- [`materials/slint-ui_docs/guide/development/best-practices.mdx`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/guide/development/best-practices.mdx)
- [`materials/slint-ui_docs/reference/std-widgets/globals/palette.mdx`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/reference/std-widgets/globals/palette.mdx)
- [`materials/slint-ui_docs/reference/common.mdx`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/reference/common.mdx)

---

## 11. What NodeChat Should Borrow First

If the new project is built in a sensible order, the first things to copy from the docs are not the fanciest widgets. The first things to copy are the structural patterns:

- responsive root shell
- explicit layout rules
- reusable page shells
- list and card models
- focus management
- modal confirmation flows
- clear navigation differences for mobile and desktop

That gives NodeChat the long-term foundation it needs before any styling polish.

Reference:

- [`nodechat-new/NEW_UI_FOUNDATION_PLAN.md`](/home/kristency/Projects/nodechat/nodechat-new/NEW_UI_FOUNDATION_PLAN.md)
- [`materials/slint-ui_docs/index.mdx`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/index.mdx)
- [`materials/slint-ui_docs/material-components/README.md`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/material-components/README.md)

---

## 12. Multiline Text Editing Is a Real UI Case, Not an Edge Case

The `TextEditPage` example is useful because it shows how the same text content can be presented with different wrapping modes side by side. That is a good pattern for NodeChat, where the same input or content may need different presentation rules depending on the screen.

What this teaches:

- multiline input is different from single-line input
- word wrap and no-wrap are both valid states depending on the task
- editable text areas should be shown inside clear containers
- read-only and enabled states should be first-class, not hacks

For NodeChat, this is relevant for:

- message composition if multiline is added
- settings or profile text editing
- diagnostics or note-like fields
- any screen where long content must remain readable on mobile

Reference:

- [`materials/slint_demos/examples-gallery/ui/pages/text_edit_page.slint`](/home/kristency/Projects/nodechat/materials/slint_demos/examples-gallery/ui/pages/text_edit_page.slint)
- [`materials/slint-ui_docs/reference/std-widgets/views/textedit.mdx`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/reference/std-widgets/views/textedit.mdx)
- [`materials/slint-ui_docs/reference/keyboard-input/textinput.mdx`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/reference/keyboard-input/textinput.mdx)

---

## 13. Mobile Keyboard Behavior Must Be Designed In

This is one of the most important missing pieces for NodeChat.

The mobile docs explain that virtual keyboards reduce usable screen space and that the app must keep the focused editing area visible by placing it inside a scrollable ancestor. They also make one very important point: nested scroll views are not the answer.

What to copy:

- put editable areas inside a scrollable region
- let the focused input stay visible when the keyboard appears
- keep only one primary scroll container around the editing area
- use the window’s safe-area and virtual-keyboard properties as part of layout thinking

What this means for NodeChat:

- the chat composer should not be allowed to disappear under the keyboard
- onboarding and password screens should remain usable when the keyboard opens
- forms should scroll into view instead of being cropped
- mobile behavior should be tested early, not after the design is already locked

This is especially relevant if the new UI project aims to be genuinely good on phones, not merely “runs on phones.”

Reference:

- [`materials/slint-ui_docs/guide/platforms/mobile/general.mdx`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/guide/platforms/mobile/general.mdx)
- [`materials/slint-ui_docs/guide/platforms/mobile/android.mdx`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/guide/platforms/mobile/android.mdx)
- [`materials/slint-ui_docs/reference/keyboard-input/textinputinterface.mdx`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/reference/keyboard-input/textinputinterface.mdx)
- [`materials/slint-ui_docs/reference/std-widgets/views/scrollview.mdx`](/home/kristency/Projects/nodechat/materials/slint-ui_docs/reference/std-widgets/views/scrollview.mdx)

---

## 14. Comparisons and Demo Pages Can Guide Layout Decisions

The gallery examples are not just demos of widgets. They are also good examples of how to compare related UI states without making the file structure messy.

The `ListViewPage` and `TextEditPage` examples show a practical technique:

- present similar controls in separate grouped regions
- keep the page wrapper stable
- vary only the content being demonstrated

For NodeChat, this suggests that build-time or prototype screens can be used to compare:

- direct chat vs group chat layout
- mobile vs desktop navigation
- editable vs read-only states
- wrapped vs single-line text behavior

That kind of comparison is useful during the rebuild because it helps define the final component boundaries before the real app screens are migrated.

Reference:

- [`materials/slint_demos/examples-gallery/ui/pages/list_view_page.slint`](/home/kristency/Projects/nodechat/materials/slint_demos/examples-gallery/ui/pages/list_view_page.slint)
- [`materials/slint_demos/examples-gallery/ui/pages/text_edit_page.slint`](/home/kristency/Projects/nodechat/materials/slint_demos/examples-gallery/ui/pages/text_edit_page.slint)
- [`materials/slint_demos/examples-gallery/ui/pages/page.slint`](/home/kristency/Projects/nodechat/materials/slint_demos/examples-gallery/ui/pages/page.slint)

---

## Practical Summary

For NodeChat, the Slint docs point to a simple conclusion:

- build the UI as a system, not a pile of screens
- keep layout responsibility in shells and shared components
- keep focus, scrolling, and responsiveness explicit
- use cards, tiles, app bars, and navigation components as structural guidance
- preserve the backend contract and rebuild the presentation layer cleanly

That is the best route if the goal is long-term maintainability and a UI that scales cleanly across mobile and desktop.
