# NodeChat UI Layout Guidelines

Goal: make every page feel consistent, responsive, and stable on desktop and mobile.

## Core Rules

- Use one responsive page pattern across the app.
- Desktop content should be left-aligned, not centered.
- Mobile content should be full width with a small fixed gutter.
- Avoid horizontal overflow on any screen.
- Keep page-specific actions inside the page body, not in the shared header.

## Page Width Rules

- Desktop body width should be capped to a comfortable reading width.
- Mobile body width should fill the screen minus side margins.
- Do not let cards or rows exceed their parent width.
- Prefer `horizontal-stretch: 1` on content containers that must fill available space.

## Alignment Rules

- Use `alignment: start` for normal content on desktop.
- Use `alignment: center` only for intentional empty states or splash-like content.
- Avoid mixed center/left alignment within the same page unless it is visually intentional.

## Overflow Rules

- Wrap or elide text that can grow.
- Keep card contents clipped if they may overrun the surface.
- Keep scrollable content inside one `ScrollView` per page area.
- Do not allow fixed widths to break mobile layout.

## Chat View Rules

- Treat chat view separately from detail pages.
- Chat view should have:
  - top header
  - message area
  - bottom composer
- The chat page should not use extra duplicate header chrome.
- The message area should be the only major scrollable region.
- New messages should keep the bottom visible when the user is already near the bottom.
- If the user scrolls upward manually, do not force them back to the bottom on every message.
- On mobile, the composer and message list should move with the keyboard so the latest messages remain reachable.

## Shared Components To Keep

- `AppHeader`
- `ConversationIdentityRow`
- `ActionRow`

## Next Implementation Order

1. Normalize body width and side margins on non-chat pages.
2. Simplify `ChatViewScreen` into a dedicated chat layout.
3. Add keyboard-safe scrolling and bottom anchoring behavior for chat.

