# Slint Demos Notes for NodeChat

This document collects the parts of `materials/slint_demos` that are actually useful for rebuilding NodeChat UI on a cleaner foundation.

The main value of the demos is not their visuals. It is the way they separate:

- root app shell
- page shells
- shared widgets
- navigation
- responsive layout variants
- scrollable content areas

That is the pattern NodeChat should borrow.

---

## 1. Root App Shells

The demos consistently keep the top-level window small and intentional. The root window decides the active mode, safe-area padding, and which major layout branch is visible. That is a better model than letting every screen manage its own frame.

What to copy:

- one root window as the layout entry point
- one place for responsive breakpoints
- one place for safe-area padding
- one place to choose mobile, mid, or desktop variants

What to avoid:

- screens deciding their own global layout
- mixing page content with app-shell logic
- hardcoding the same window behavior in many files

For NodeChat, this means the shell should be responsible for mobile/desktop switching, overlays, and global spacing. Individual screens should only render content.

Reference:

- [`materials/slint_demos/energy-monitor/ui/desktop_window.slint`](/home/kristency/Projects/nodechat/materials/slint_demos/energy-monitor/ui/desktop_window.slint)
- [`materials/slint_demos/weather-demo/ui/main.slint`](/home/kristency/Projects/nodechat/materials/slint_demos/weather-demo/ui/main.slint)
- [`materials/slint_demos/examples-gallery/ui/pages/page.slint`](/home/kristency/Projects/nodechat/materials/slint_demos/examples-gallery/ui/pages/page.slint)

---

## 2. Responsive Variants

The strongest demo pattern is explicit responsive branching. The energy monitor demo does not pretend one layout can serve every screen size equally well. It chooses separate components for desktop, medium, small, and mobile presentations.

What to copy:

- explicit screen-size selection
- dedicated mobile layout component
- dedicated desktop layout component
- small reusable pieces that can be recomposed per breakpoint

Why it matters:

- mobile chat UIs need touch-safe spacing and full-width panels
- desktop chat UIs need persistent sidebars and denser information
- using the same component tree for both usually creates compromise UI

For NodeChat, the main app should likely choose between:

- a mobile stacked layout
- a desktop two-panel layout

Reference:

- [`materials/slint_demos/energy-monitor/ui/desktop_window.slint`](/home/kristency/Projects/nodechat/materials/slint_demos/energy-monitor/ui/desktop_window.slint)
- [`materials/slint_demos/energy-monitor/ui/big_main.slint`](/home/kristency/Projects/nodechat/materials/slint_demos/energy-monitor/ui/big_main.slint)
- [`materials/slint_demos/energy-monitor/ui/mobile_main.slint`](/home/kristency/Projects/nodechat/materials/slint_demos/energy-monitor/ui/mobile_main.slint)
- [`materials/slint_demos/weather-demo/ui/page-base.slint`](/home/kristency/Projects/nodechat/materials/slint_demos/weather-demo/ui/page-base.slint)

---

## 3. Page Shells

Several demos use a small page wrapper that adds consistent padding, size, and transition behavior around content. That is exactly the kind of structure NodeChat needs for chat list, contacts, settings, and detail pages.

What to copy:

- a reusable page wrapper
- one place for horizontal padding
- one place for page transition behavior
- one consistent layout skeleton for page content

Why it helps:

- page files stay smaller
- visual spacing becomes consistent
- mobile and desktop bodies remain aligned
- page reuse becomes easier

For NodeChat, a `PageShell` or `DetailPageShell` would help reduce the repeated layout code that is currently scattered across screens.

Reference:

- [`materials/slint_demos/energy-monitor/ui/pages/page.slint`](/home/kristency/Projects/nodechat/materials/slint_demos/energy-monitor/ui/pages/page.slint)
- [`materials/slint_demos/examples-gallery/ui/pages/page.slint`](/home/kristency/Projects/nodechat/materials/slint_demos/examples-gallery/ui/pages/page.slint)
- [`materials/slint_demos/weather-demo/ui/page-base.slint`](/home/kristency/Projects/nodechat/materials/slint_demos/weather-demo/ui/page-base.slint)

---

## 4. Scrollable Content

The demos show that scroll behavior should be explicit, not accidental. The energy monitor scroll container and the gallery list patterns are useful because they treat scrolling as part of the layout design, not as a side effect.

What to copy:

- one primary scroll area per page
- scroll containers that are designed around content size
- deliberate selection and focus handling inside scroll regions
- card/list items that stay inside their container width

Why it matters for NodeChat:

- chat history must scroll cleanly on mobile
- long contact lists should remain usable
- detail pages must not overflow horizontally
- keyboard changes should not break the visible region

NodeChat should treat the message list as the main scrollable area in chat view, with the composer anchored separately.

Reference:

- [`materials/slint_demos/energy-monitor/ui/widgets/page_scroll_view.slint`](/home/kristency/Projects/nodechat/materials/slint_demos/energy-monitor/ui/widgets/page_scroll_view.slint)
- [`materials/slint_demos/usecases/ui/widgets/card_list_view.slint`](/home/kristency/Projects/nodechat/materials/slint_demos/usecases/ui/widgets/card_list_view.slint)
- [`materials/slint_demos/examples-gallery/ui/pages/list_view_page.slint`](/home/kristency/Projects/nodechat/materials/slint_demos/examples-gallery/ui/pages/list_view_page.slint)

---

## 5. Card and List Patterns

The usecases demo is especially relevant because it shows how to build list items as cards with consistent padding, selected states, and text hierarchy.

What to copy:

- card-based list rows
- title, subtitle, note, caption hierarchy
- selected-state styling that is centralized
- reusable list item components instead of ad hoc row markup

Why it helps NodeChat:

- chat rows, contacts, and action rows can share the same visual discipline
- list cards will feel consistent across the app
- text truncation and wrapping become easier to standardize

For NodeChat, this suggests a shared row/card system for:

- chat previews
- contacts
- group members
- conversation actions
- settings entries

Reference:

- [`materials/slint_demos/usecases/ui/widgets/card_list_view.slint`](/home/kristency/Projects/nodechat/materials/slint_demos/usecases/ui/widgets/card_list_view.slint)
- [`materials/slint_demos/printerdemo/ui/components/headers.slint`](/home/kristency/Projects/nodechat/materials/slint_demos/printerdemo/ui/components/headers.slint)
- [`materials/slint_demos/printerdemo/ui/components/sidebar.slint`](/home/kristency/Projects/nodechat/materials/slint_demos/printerdemo/ui/components/sidebar.slint)

---

## 6. Headers and Navigation Chrome

The printer demo shows a clean pattern for reusable headers: one back header, one plain header, and a consistent separator. The energy monitor demo shows a simpler mobile header. The usecases demo keeps header logic separate from content.

What to copy:

- dedicated header components
- header variants instead of duplicating the whole header
- a back button in one predictable place
- title and subtitle separation

Why it matters:

- NodeChat currently has too many page-specific header variants
- one reusable app header would reduce visual drift
- a conversation header can be separate from a normal page header

For NodeChat, this points toward:

- `AppHeader`
- `ConversationHeader`
- `MobileHeader`
- `DetailHeader`

Reference:

- [`materials/slint_demos/printerdemo/ui/components/headers.slint`](/home/kristency/Projects/nodechat/materials/slint_demos/printerdemo/ui/components/headers.slint)
- [`materials/slint_demos/energy-monitor/ui/blocks/mobile_header.slint`](/home/kristency/Projects/nodechat/materials/slint_demos/energy-monitor/ui/blocks/mobile_header.slint)
- [`materials/slint_demos/usecases/ui/views/header_view.slint`](/home/kristency/Projects/nodechat/materials/slint_demos/usecases/ui/views/header_view.slint)

---

## 7. Sidebar and Navigation

The printer demo sidebar is a useful example of persistent navigation with state-driven active items, icon buttons, and a theme popup. The energy monitor demo also shows navigation as a separate component rather than something embedded in every page.

What to copy:

- navigation as its own component
- active-state driven icon items
- a clear separation between navigation and content
- optional overlay or menu behavior attached to the shell, not every screen

Why it matters for NodeChat:

- desktop should probably use a persistent left rail or chat list panel
- mobile should probably use a smaller navigation model or bottom/stacked navigation
- screen switching should not force each page to reimplement its own navigation chrome

Reference:

- [`materials/slint_demos/printerdemo/ui/components/sidebar.slint`](/home/kristency/Projects/nodechat/materials/slint_demos/printerdemo/ui/components/sidebar.slint)
- [`materials/slint_demos/energy-monitor/ui/widgets/navigation.slint`](/home/kristency/Projects/nodechat/materials/slint_demos/energy-monitor/ui/widgets/navigation.slint)
- [`materials/slint_demos/printerdemo/ui/components/icon_button.slint`](/home/kristency/Projects/nodechat/materials/slint_demos/printerdemo/ui/components/icon_button.slint)

---

## 8. Focus and Page Transitions

The weather demo is useful because it shows a proper transition model with focus handling. It explicitly manages page opening and closing, and it clears focus when dismissing overlays or changing pages.

What to copy:

- explicit page open/close state
- focus clearing when leaving a screen
- transition animations tied to state changes
- preventing stale focus from leaking across pages

Why it matters for NodeChat:

- this directly addresses the input focus problem you already noticed
- mobile keyboards and stale cursor state are usually caused by weak focus management
- overlays and page switches need a clear focus rule

For NodeChat, focus should be cleared when:

- a page changes
- a modal closes
- a chat composer loses relevance
- the lock/onboarding state changes

Reference:

- [`materials/slint_demos/weather-demo/ui/main.slint`](/home/kristency/Projects/nodechat/materials/slint_demos/weather-demo/ui/main.slint)
- [`materials/slint_demos/weather-demo/ui/controls/stackview.slint`](/home/kristency/Projects/nodechat/materials/slint_demos/weather-demo/ui/controls/stackview.slint)
- [`materials/slint_demos/weather-demo/ui/controls/busy-layer.slint`](/home/kristency/Projects/nodechat/materials/slint_demos/weather-demo/ui/controls/busy-layer.slint)

---

## 9. Shared Styling and Token Systems

The demos consistently push colors, spacing, and typography into shared style modules. That is one of the most important things to borrow.

What to copy:

- theme tokens in one place
- palette decisions centralized
- spacing constants instead of random numbers everywhere
- reusable component theme data

Why it matters for NodeChat:

- the UI will look coherent
- mobile and desktop will use the same language
- future screens can be added without redesigning the whole app

For NodeChat, the best long-term move is a small design system layer first, then screens built on top of it.

Reference:

- [`materials/slint_demos/energy-monitor/ui/theme.slint`](/home/kristency/Projects/nodechat/materials/slint_demos/energy-monitor/ui/theme.slint)
- [`materials/slint_demos/weather-demo/ui/style/styles.slint`](/home/kristency/Projects/nodechat/materials/slint_demos/weather-demo/ui/style/styles.slint)
- [`materials/slint_demos/printerdemo/ui/common.slint`](/home/kristency/Projects/nodechat/materials/slint_demos/printerdemo/ui/common.slint)
- [`materials/slint_demos/usecases/ui/widgets/styling.slint`](/home/kristency/Projects/nodechat/materials/slint_demos/usecases/ui/widgets/styling.slint)

---

## 10. What This Means for NodeChat

The biggest lesson from the demos is structural:

- build a shell first
- split mobile and desktop intentionally
- centralize reusable layout pieces
- keep scroll areas and headers consistent
- keep state out of individual screens where possible

For NodeChat, that means the new UI should probably be organized as:

- root shell
- responsive layout switch
- shared page shell
- shared headers
- shared cards and rows
- chat-specific shell
- detail-page shell

That is a much cleaner base than continuing to extend the current packed UI.

Reference:

- [`materials/slint_demos/README.md`](/home/kristency/Projects/nodechat/materials/slint_demos/README.md)
- [`NEW_UI_FOUNDATION_PLAN.md`](/home/kristency/Projects/nodechat/NEW_UI_FOUNDATION_PLAN.md)
- [`materials/slint_demos/usecases/ui/views/main_view.slint`](/home/kristency/Projects/nodechat/materials/slint_demos/usecases/ui/views/main_view.slint)
