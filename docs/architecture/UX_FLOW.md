# NodeChat — UX Flow & Interface Design

**Document Type:** UX Flow Reference
**Project:** NodeChat — Secure Decentralized Chat
**Covers:** First Launch · Returning User · Mobile Layout · Desktop Layout · All Screens

---

## Table of Contents

1. [Design Philosophy](#1-design-philosophy)
2. [First Launch Flow (Both Platforms)](#2-first-launch-flow-both-platforms)
3. [Returning User Flow (Both Platforms)](#3-returning-user-flow-both-platforms)
4. [Mobile Layout & Navigation](#4-mobile-layout--navigation)
5. [Desktop Layout & Navigation](#5-desktop-layout--navigation)
6. [Screen-by-Screen Reference](#6-screen-by-screen-reference)
7. [Shared UI Components](#7-shared-ui-components)
8. [Slint Implementation Notes](#8-slint-implementation-notes)
9. [UX Decisions & Rationale](#9-ux-decisions--rationale)

---

## 1. Design Philosophy

NodeChat's visual design follows three principles:

**1. Honest UI.** The interface never lies about delivery state, connection health, or encryption status. If a message is queued because a peer is offline, it says "queued." If the node is connected through a relay instead of directly, it says so. Users are treated as adults who can handle reality.

**2. Familiar shell, different guts.** The surface layout intentionally resembles WhatsApp/Telegram — left panel contacts, right panel messages, input bar at the bottom. The difference is what's under that shell: no phone number, no account, no central server. Familiarity lowers the learning curve. The cryptographic reality surfaces only when relevant.

**3. Mobile-first, desktop-enhanced.** The core chat experience is designed for mobile first. The desktop layout takes the same flows and adds a persistent two-panel layout because screen space allows it — not because desktop gets different features.

### Colour Tokens (defined once in `ui/app.slint`, used everywhere)

| Token | Light | Dark | Usage |
|---|---|---|---|
| `surface-primary` | `#FFFFFF` | `#1C1C1E` | Main backgrounds |
| `surface-secondary` | `#F2F2F7` | `#2C2C2E` | Sidebar, input areas |
| `surface-tertiary` | `#E5E5EA` | `#3A3A3C` | Cards, panels |
| `accent` | `#1A5FA8` | `#4A9EE8` | Sent bubbles, buttons, links |
| `accent-success` | `#2D9B5F` | `#34C774` | Online indicators, verified badges |
| `accent-warning` | `#C97B1A` | `#F4A623` | Relay mode, unverified badges |
| `accent-danger` | `#C0392B` | `#FF453A` | Errors, destructive actions |
| `text-primary` | `#000000` | `#FFFFFF` | Main text |
| `text-secondary` | `#6C6C70` | `#8E8E93` | Timestamps, subtitles |
| `text-tertiary` | `#AEAEB2` | `#636366` | Placeholders, disabled |
| `bubble-out` | `#1A5FA8` | `#1A5FA8` | Outgoing message bubbles |
| `bubble-in` | `#FFFFFF` | `#2C2C2E` | Incoming message bubbles |

These tokens are declared as global properties in `ui/app.slint` and referenced by name across all component files — never hardcoded.

---

## 2. First Launch Flow (Both Platforms)

The first-launch flow runs exactly once. After completion, it never appears again unless the user wipes app data. The goal is to get the user from zero to a working identity with a NodeId they can share — in under 60 seconds, with no account creation.

### Screen 2.1 — Welcome Screen

```
┌─────────────────────────────────┐
│                                 │
│                                 │
│           [App Icon]            │
│                                 │
│           NodeChat              │
│   Secure. Private. Yours.       │
│                                 │
│                                 │
│                                 │
│   ┌─────────────────────────┐   │
│   │      Get Started        │   │
│   └─────────────────────────┘   │
│                                 │
│   No account. No phone number.  │
│   No server.                    │
│                                 │
└─────────────────────────────────┘
```

**What it shows:** App name, tagline, single "Get Started" CTA, one-liner reassurance.

**Background:** Nothing. The keypair is NOT generated here — we wait until the user confirms their name so identity generation feels intentional.

**Slint file:** `ui/screens/welcome.slint`
**Transitions to:** Screen 2.2

---

### Screen 2.2 — Choose Your Display Name

```
┌─────────────────────────────────┐
│  ←   Set up your identity       │
│─────────────────────────────────│
│                                 │
│  What should people call you?   │
│                                 │
│  ┌─────────────────────────┐    │
│  │  Display Name           │    │
│  └─────────────────────────┘    │
│                                 │
│  This name is only shared       │
│  with people you contact.       │
│  It is never sent to any server.│
│                                 │
│   ┌─────────────────────────┐   │
│   │       Continue          │   │  ← disabled until valid input
│   └─────────────────────────┘   │
│                                 │
└─────────────────────────────────┘
```

**Validation:** 1–32 characters. Unicode letters, numbers, spaces, basic punctuation only.
"Continue" is disabled until valid input is provided — enforced via a Slint `enabled:` binding on the button.

**Slint file:** `ui/screens/setup_name.slint`
**Transitions to:** Screen 2.3

---

### Screen 2.3 — Generating Identity (Loading)

```
┌─────────────────────────────────┐
│                                 │
│                                 │
│        [Spinner / Progress]     │
│                                 │
│     Generating your identity    │
│                                 │
│   Your cryptographic keypair    │
│   is being created locally.     │
│   This happens once, on this    │
│   device, entirely offline.     │
│                                 │
│                                 │
└─────────────────────────────────┘
```

**Backend sequence:**
1. X25519/Ed25519 keypair generated
2. Public key derived → becomes `NodeId`
3. Identity written to local SQLite
4. Iroh endpoint binds and begins listening
5. Pkarr publishes the user's NodeAddr

**Duration:** Minimum 1.5 seconds shown regardless of actual time — long enough to read, short enough to not feel broken.

**Slint file:** `ui/screens/setup_name.slint` (loading state variant via `in-out property <bool> generating`)
**Transitions to:** Screen 2.4

---

### Screen 2.4 — Your NodeId (Identity Card)

```
┌─────────────────────────────────┐
│  ←   Your Identity              │
│─────────────────────────────────│
│                                 │
│         Hi, Shedrack!           │
│     Your identity is ready.     │
│                                 │
│  ┌─────────────────────────┐    │
│  │                         │    │
│  │      [QR CODE]          │    │
│  │                         │    │
│  └─────────────────────────┘    │
│                                 │
│  Your Node ID:                  │
│  ┌─────────────────────────┐    │
│  │ nBq3...Kx7R  [Copy]     │    │
│  └─────────────────────────┘    │
│                                 │
│  Share this with anyone you     │
│  want to chat with.             │
│                                 │
│   ┌─────────────────────────┐   │
│   │      Go to Chats        │   │
│   └─────────────────────────┘   │
│                                 │
└─────────────────────────────────┘
```

**Slint file:** `ui/screens/identity_card.slint`
**Transitions to:** Screen 2.5 (optional password) OR Main App

---

### Screen 2.5 — Set Local Password (Optional)

```
┌─────────────────────────────────┐
│  ←   Protect your data          │
│─────────────────────────────────│
│                                 │
│  Lock NodeChat with a password  │
│                                 │
│  If your device is stolen, your │
│  messages and identity cannot   │
│  be accessed without it.        │
│                                 │
│  ┌─────────────────────────┐    │
│  │  Password               │    │
│  └─────────────────────────┘    │
│                                 │
│  ┌─────────────────────────┐    │
│  │  Confirm Password       │    │
│  └─────────────────────────┘    │
│                                 │
│   ┌─────────────────────────┐   │
│   │      Set Password       │   │
│   └─────────────────────────┘   │
│                                 │
│         Skip for now →          │
│                                 │
└─────────────────────────────────┘
```

**Notes:** Skipping is explicitly allowed. Password encrypts the SQLite file and private key on disk. Never transmitted anywhere. Configurable again in Settings.

**Slint file:** `ui/screens/settings.slint` (onboarding variant)
**Transitions to:** Main App

---

## 3. Returning User Flow (Both Platforms)

### Screen 3.1A — Password Gate (if password was set)

```
┌─────────────────────────────────┐
│                                 │
│           [App Icon]            │
│           NodeChat              │
│                                 │
│  ┌─────────────────────────┐    │
│  │  Enter Password         │    │
│  └─────────────────────────┘    │
│                                 │
│   ┌─────────────────────────┐   │
│   │         Unlock          │   │
│   └─────────────────────────┘   │
│                                 │
│   Use biometrics instead  →     │  ← mobile only
│                                 │
└─────────────────────────────────┘
```

**Behaviour:**
- Wrong password: shake animation on input field (Slint `@keyframe` animation), "Incorrect password" label appears, input clears
- 5 consecutive wrong attempts: 30-second cooldown with a visible countdown timer
- Correct password: SQLite decrypts, private key loads, Iroh endpoint binds, Pkarr re-publishes, queued messages begin flushing

**Slint file:** `ui/screens/welcome.slint` (locked state variant)

### Screen 3.1B — Direct Launch (no password)

App loads directly into the main UI. Same backend bootstrap fires without the decryption step.

### Screen 3.2 — Reconnecting State

- Main UI loads immediately with cached SQLite state
- Subtle "Connecting…" label in the status bar
- Peer online indicators update live as connections are established
- Queued messages flush silently in the background

No loading screen. The user sees their chats immediately.

---

## 4. Mobile Layout & Navigation

Mobile uses a **stack-based navigation model** — one screen at a time, back button to go up. No persistent side panels. This matches the navigation pattern Android users expect from every messaging app.

### 4.1 Mobile Home Screen (Chat List)

```
┌───────────────────────────────────┐
│  NodeChat          [•] [+] [≡]   │
│───────────────────────────────────│
│  🔍  Search chats…                │
│───────────────────────────────────│
│                                   │
│  ┌───────────────────────────┐    │
│  │ [AV]  Eric O.         12:41│   │
│  │       Sent the ref doc ✓✓ │   │
│  └───────────────────────────┘    │
│                                   │
│  ┌───────────────────────────┐    │
│  │ [AV]  Mama           09:12│   │
│  │       ⏳ 2 msgs queued    │   │
│  └───────────────────────────┘    │
│                                   │
│  ┌───────────────────────────┐    │
│  │ [GR]  Project Group   Tue │   │
│  │       You: Let's ship it  │   │
│  └───────────────────────────┘    │
│                                   │
│  ┌───────────────────────────┐    │
│  │ [AV]  Anon 4f7c…     Mon │   │
│  │ ⚠️    Key not verified    │   │
│  └───────────────────────────┘    │
│                                   │
└───────────────────────────────────┘
│  [Chats]   [Contacts]  [Settings] │
└───────────────────────────────────┘
```

**`[•]` Network status dot:**
- Green: direct P2P connected
- Amber: connected via relay (reduced privacy)
- Red: offline — all messages queuing

**Chat row:** Avatar (initials), display name, timestamp, last message preview or queue status, delivery ticks, unread count badge.

**Tab bar:** Chats (default) · Contacts · Settings

**Slint files:** `ui/screens/chat_list.slint`, `ui/components/contact_row.slint`

---

### 4.2 Mobile Chat Screen (1:1)

```
┌───────────────────────────────────┐
│  ←  [AV]  Eric O.          [···] │
│       0xAF3B…C72E · direct · 🔒  │
│───────────────────────────────────│
│                                   │
│          Tuesday, 29 Mar          │
│                                   │
│  ┌─────────────────────────┐      │
│  │ yo I pushed the ref doc │      │
│  │ to the repo             │      │
│  │                   12:38 │      │
│  └─────────────────────────┘      │
│                                   │
│      ┌─────────────────────────┐  │
│      │ received it, reviewing  │  │
│      │ phase 2 now             │  │
│      │ 12:40              ✓✓  │  │
│      └─────────────────────────┘  │
│                                   │
│      ┌─────────────────────────┐  │
│      │ ⏱ disappears in 24h    │  │
│      │ what about adding       │  │
│      │ supabase?               │  │
│      │ 12:41           routing…│  │
│      └─────────────────────────┘  │
│                                   │
│  --- Key ratcheted · session #85  │
│                                   │
│───────────────────────────────────│
│  [📎]  Type a message…    [Send]  │
└───────────────────────────────────┘
```

**Header:** Back · Avatar (tappable → contact info) · Key fingerprint · Connection mode · 🔒 (tappable → E2EE info sheet) · `[···]` (options menu)

**Delivery states on outgoing bubbles:**
- `sending…` — encrypting and dispatching
- `routing…` — propagating through network
- `✓` — sent from this device
- `✓✓` — confirmed received by recipient node
- `⏳ queued` — recipient offline, stored locally

**System event rows (muted, centered):**
- `--- Session started · Forward Secrecy active ---`
- `--- Key ratcheted · session #85 ---`
- `--- Eric verified your key ---`

**Slint files:** `ui/screens/chat_view.slint`, `ui/components/message_bubble.slint`, `ui/components/chat_input.slint`

---

### 4.3 Mobile Chat Screen (Group)

```
┌───────────────────────────────────┐
│  ←  [GR]  Project Group    [···] │
│       3 members · gossip swarm    │
│───────────────────────────────────│
│                                   │
│  ┌─────────────────────────┐      │
│  │ Eric O.                 │      │
│  │ pushed the build        │      │
│  │                   15:02 │      │
│  └─────────────────────────┘      │
│                                   │
│      ┌─────────────────────────┐  │
│      │ looks good from my end  │  │
│      │ 15:10               ✓✓ │  │
│      └─────────────────────────┘  │
│                                   │
│───────────────────────────────────│
│  [📎]  Type a message…    [Send]  │
└───────────────────────────────────┘
```

**Differences from 1:1:** Sender name above each incoming bubble. Header shows member count and `gossip swarm`. No per-message delivery receipts — gossip is broadcast with no per-recipient ACK.

**Slint file:** `ui/screens/group_view.slint`

---

### 4.4 Mobile Add Contact Flow

```
┌───────────────────────────────────┐
│  ←   Add Contact                  │
│───────────────────────────────────│
│                                   │
│  ┌───────────────────────────┐    │
│  │    📷  Scan their QR code │    │
│  └───────────────────────────┘    │
│                                   │
│  ┌───────────────────────────┐    │
│  │    ⌨️   Enter Node ID     │    │
│  └───────────────────────────┘    │
│                                   │
└───────────────────────────────────┘
```

**QR path:** Camera opens → scans NodeId → "Add Shedrack G.?" confirmation → added to phonebook, connection attempt fires.

**Manual path:** Paste NodeId → assign local name → Add Contact → immediate connection attempt.

**Slint file:** `ui/screens/add_contact.slint`

---

### 4.5 Mobile Key Verification Screen

```
┌───────────────────────────────────┐
│  ←   Verify Eric O.               │
│───────────────────────────────────│
│                                   │
│  Compare these numbers with Eric  │
│  in person or via another channel.│
│                                   │
│  ┌────────┬────────┬────────┐     │
│  │ 47821  │ 90134  │ 22871  │     │
│  │ 55409  │ 13782  │ 88021  │     │
│  │ 30156  │ 71943  │ 04820  │     │
│  └────────┴────────┴────────┘     │
│                                   │
│  ┌─────────────────────────────┐  │
│  │       Scan QR instead       │  │
│  └─────────────────────────────┘  │
│                                   │
│  ┌─────────────────────────────┐  │
│  │   ✓  Mark as Verified       │  │
│  └─────────────────────────────┘  │
│                                   │
│       Numbers don't match?        │
│  Someone may be intercepting.     │
│  Do not continue.                 │
│                                   │
└───────────────────────────────────┘
```

On verification: green `Verified` badge in chat header and contact list. System message in chat: `--- You verified Eric's identity ---`.

**Slint file:** `ui/screens/verify_key.slint`

---

### 4.6 Mobile Settings Screen

```
┌───────────────────────────────────┐
│  ←   Settings                     │
│───────────────────────────────────│
│  IDENTITY                         │
│  Display Name: Shedrack G.        │
│  Node ID: nBq3…Kx7R  [↗] [QR]   │
│                                   │
│  SECURITY                         │
│  App Password                 [>] │
│  Biometric Unlock             [>] │
│                                   │
│  PRIVACY                          │
│  Default ephemeral timer  [Off ▾] │
│  Read receipts         [ON ●]     │
│                                   │
│  NETWORK                          │
│  8 peers · 1 via relay            │
│  Node address: [View]             │
│                                   │
│  DANGER ZONE                      │
│  Clear all messages           [>] │
│  Delete identity              [>] │
│                                   │
└───────────────────────────────────┘
```

**Slint file:** `ui/screens/settings.slint`

---

## 5. Desktop Layout & Navigation

Desktop uses a **persistent two-panel layout**. Contacts panel always visible on the left. Active chat fills the right panel. No navigation stack — panels update in place.

### 5.1 Desktop — Empty State

```
┌──────────────────────────────────────────────────────────────────┐
│  NodeChat    [•] Connected · 8 peers · 1 relay      [≡] Settings │
├──────────────┬───────────────────────────────────────────────────┤
│  🔍 Search…  │                                                   │
│──────────────│         Select a chat to start messaging.         │
│  Eric O.     │                                                   │
│  ✓  12:41 ✓✓│                                                   │
│              │                                                   │
│  Mama        │                                                   │
│  ⏳ Queued   │                                                   │
│              │                                                   │
│  Project Grp │                                                   │
│  15:04       │                                                   │
│──────────────│                                                   │
│  [+] New     │                                                   │
│  [⊕] Group   │                                                   │
└──────────────┴───────────────────────────────────────────────────┘
```

**Left panel (~260px fixed):** App title bar with global network status. Search. Scrollable contact + group list sorted by last activity. New chat and new group buttons at the bottom.

**Right panel (fills remaining width):** Empty state until a chat is selected.

---

### 5.2 Desktop — Active Chat (1:1)

```
┌──────────────────────────────────────────────────────────────────┐
│  NodeChat    [•] Connected · 8 peers · 1 relay      [≡] Settings │
├──────────────┬───────────────────────────────────────────────────┤
│  🔍 Search…  │  [AV] Eric O.                 [Verify] [···]      │
│──────────────│  0xAF3B…C72E · direct · 🔒                        │
│  ► Eric O.   │  ─────────────────────────────────────────────────│
│  ✓  12:41 ✓✓│                                                   │
│              │  ┌────────────────────────────┐                   │
│  Mama        │  │ yo I pushed the ref doc    │                   │
│  ⏳ Queued   │  │                      12:38 │                   │
│              │  └────────────────────────────┘                   │
│  Project Grp │                                                   │
│  15:04       │          ┌────────────────────────────┐           │
│              │          │ received it, reviewing     │           │
│              │          │ 12:40                  ✓✓ │           │
│              │          └────────────────────────────┘           │
│              │                                                   │
│              │  --- Key ratcheted · session #85 ---              │
│              │  ─────────────────────────────────────────────────│
│  [+] New     │  [📎]  Type a message…                    [Send]  │
│  [⊕] Group   │                                                   │
└──────────────┴───────────────────────────────────────────────────┘
```

**Desktop-only additions:**
- `[Verify]` button always visible in the chat header (not buried in a menu)
- Full key fingerprint and connection mode persistent in the subheader
- `Enter` sends. `Shift+Enter` for newlines.
- `Ctrl+K` focuses the search box in the left panel

---

### 5.3 Desktop — Settings Panel

Settings open as the right panel replacing the chat view. Contact list stays visible.

```
┌──────────────────────────────────────────────────────────────────┐
│  NodeChat    [•] Connected                          [≡] Settings │
├──────────────┬───────────────────────────────────────────────────┤
│  🔍 Search…  │  Settings                              [✕ Close]  │
│──────────────│  ─────────────────────────────────────────────────│
│  Eric O.     │  IDENTITY                                         │
│  Mama        │  Display Name    Shedrack G.            [Edit]    │
│  Project Grp │  Node ID         nBq3…Kx7R    [Copy] [Show QR]   │
│              │                                                   │
│              │  SECURITY                                         │
│              │  App Password                          [Change]   │
│              │                                                   │
│              │  PRIVACY                                          │
│              │  Default ephemeral timer            [Off     ▾]   │
│              │  Read receipts                    [●────── ON]    │
│              │                                                   │
│              │  NETWORK STATUS                                   │
│              │  Peers: 8 direct · 1 via relay                    │
│              │  Health: ████████░░ 80%                           │
│              │  Your node address: [View full]                   │
│              │                                                   │
│  [+] New     │  ─────────────────────────────────────────────────│
│  [⊕] Group   │  DANGER ZONE  [Clear messages]  [Delete identity] │
└──────────────┴───────────────────────────────────────────────────┘
```

---

### 5.4 Desktop — Add Contact Panel

Opens as the right panel, not a blocking modal.

```
┌──────────────────────────────────────────────────────────────────┐
│  NodeChat    [•] Connected                          [≡] Settings │
├──────────────┬───────────────────────────────────────────────────┤
│  🔍 Search…  │  Add Contact                           [✕ Close]  │
│──────────────│                                                   │
│  Eric O.     │  Paste their Node ID:                             │
│  Mama        │  ┌──────────────────────────────────────────┐     │
│  Project Grp │  │  nBq3...Kx7R                        [📋] │     │
│              │  └──────────────────────────────────────────┘     │
│              │                                                   │
│              │  Give them a local name:                          │
│              │  ┌──────────────────────────────────────────┐     │
│              │  │  e.g. Eric                               │     │
│              │  └──────────────────────────────────────────┘     │
│              │                                                   │
│              │  ─── OR ───                                       │
│              │                                                   │
│              │  ┌──────────────────────────────────────────┐     │
│              │  │  📷  Scan QR Code (via webcam)           │     │
│              │  └──────────────────────────────────────────┘     │
│              │                                                   │
│              │  ┌──────────────────────────────────────────┐     │
│              │  │            Add Contact                   │     │
│              │  └──────────────────────────────────────────┘     │
│  [+] New     │                                                   │
│  [⊕] Group   │                                                   │
└──────────────┴───────────────────────────────────────────────────┘
```

---

## 6. Screen-by-Screen Reference

### First Launch Screens (run once)
| ID | Screen | Slint File | Trigger |
|---|---|---|---|
| FL-01 | Welcome | `screens/welcome.slint` | App first install |
| FL-02 | Set Display Name | `screens/setup_name.slint` | After FL-01 |
| FL-03 | Generating Identity | `screens/setup_name.slint` (loading state) | After FL-02 confirmed |
| FL-04 | Your NodeId / QR | `screens/identity_card.slint` | After keypair generated |
| FL-05 | Set Local Password (optional) | `screens/settings.slint` (onboarding state) | After FL-04 |

### Returning User Screens
| ID | Screen | Slint File | Trigger |
|---|---|---|---|
| RU-01 | Password Gate | `screens/welcome.slint` (locked state) | App launch, password set |
| RU-01B | Direct Load | — | App launch, no password |

### Main Navigation
| ID | Screen | Slint File | Platform |
|---|---|---|---|
| MN-01 | Chat List | `screens/chat_list.slint` | Mobile (home tab) |
| MN-02 | Contacts List | `screens/contacts.slint` | Mobile (contacts tab) / Desktop (left panel) |
| MN-03 | Settings | `screens/settings.slint` | Mobile (settings tab) / Desktop (right panel) |

### Chat Screens
| ID | Screen | Slint File | Trigger |
|---|---|---|---|
| CH-01 | 1:1 Chat | `screens/chat_view.slint` | Tap a direct contact |
| CH-02 | Group Chat | `screens/group_view.slint` | Tap a group |
| CH-03 | Chat Options Sheet | inline component in `chat_view.slint` | Tap `[···]` |
| CH-04 | Key Verification | `screens/verify_key.slint` | Chat options → Verify |
| CH-05 | File Transfer | inline in `chat_view.slint` | Tap `[📎]` |
| CH-06 | Ephemeral Timer Config | inline popup in `chat_view.slint` | Chat options → Set timer |

### Contact Screens
| ID | Screen | Slint File | Trigger |
|---|---|---|---|
| CO-01 | Add Contact | `screens/add_contact.slint` | Tap `[+]` |
| CO-02 | Contact Info Sheet | inline popup in `chat_view.slint` | Tap avatar in chat |
| CO-03 | New Group | inline in `contacts.slint` | Tap `[⊕]` |
| CO-04 | Group Info / Members | inline popup in `group_view.slint` | Chat options → Members |
| CO-05 | Invite to Group | inline in `group_view.slint` | Group info → Invite |

### System Screens
| ID | Screen | Slint File | Trigger |
|---|---|---|---|
| SY-01 | Network Status Detail | `components/status_dot.slint` popup | Tap status dot |
| SY-02 | E2EE Info Sheet | inline popup in `chat_view.slint` | Tap 🔒 |
| SY-03 | Danger Zone Confirm | inline dialog in `settings.slint` | Settings → destructive action |

---

## 7. Shared UI Components

### 7.1 Message Bubble (`ui/components/message_bubble.slint`)

| State | Visual |
|---|---|
| Sending | Bubble visible, reduced opacity, `sending…` in place of timestamp |
| Routing | Full opacity, `routing…` italic |
| Sent | `✓` |
| Delivered | `✓✓` |
| Queued | `⏳ queued` amber tag |
| Ephemeral | `⏱` timer tag at top; red when < 1 hour remaining |

Slint properties: `in property <string> text`, `in property <bool> is-mine`, `in property <string> status`, `in property <bool> is-ephemeral`, `in property <int> ttl-seconds`.

### 7.2 Contact Row (`ui/components/contact_row.slint`)

Always shows: Avatar (initials, background colour derived deterministically from NodeId). Display name. Verification badge (`✓ Verified` green / `⚠ Unverified` amber). Last message preview or queue status. Timestamp. Unread count badge. Online indicator dot (green direct / amber relay / gray offline).

### 7.3 Network Status Dot (`ui/components/status_dot.slint`)

Three states:
- Green: at least one direct peer
- Amber: all peers via relay (reduced privacy)
- Red: offline

Tap/click opens a popup:
```
Direct peers:     7
Relay peers:      1  (reduced privacy)
Queued messages:  2
Node uptime:      4m 32s
Your address:     [View full NodeAddr]
```

### 7.4 System Event Row (inline in `chat_view.slint`)

```
─────  Session started · Forward Secrecy active  ─────
```
Centered, `text-tertiary` colour, no interaction, no avatar, no timestamp.

### 7.5 E2EE Info Sheet (inline popup in `chat_view.slint`)

```
🔒  End-to-End Encrypted

Messages are encrypted before leaving your device.
Only you and Eric can read them.

Encryption:       ChaCha20-Poly1305
Key exchange:     X25519 DH
Forward secrecy:  Hash ratchet
Connection:       Direct P2P

Eric's fingerprint: 0xAF3B C72E 991D 04F2 ...

[ Verify key with Eric → ]
```

---

## 8. Slint Implementation Notes

These notes apply to everyone writing `.slint` files in `ui/`.

**SL-01. Colour tokens are global constants in `ui/app.slint`.**
Never hardcode hex values in component files. Always reference the global tokens: `AppTheme.accent`, `AppTheme.surface-primary`, etc.

**SL-02. Every screen is a separate `.slint` component file.**
One screen = one file in `ui/screens/`. Every reusable piece = one file in `ui/components/`. `ui/app.slint` imports them all and routes between them via a `current-screen` property.

**SL-03. Screen routing is controlled from Rust.**
The `.slint` root defines an `in-out property <int> current-screen` (or an enum equivalent). Rust code in `src/ui/mod.rs` sets this property to navigate. `.slint` files do not navigate themselves.

**SL-04. Callbacks are named clearly and documented in the `.slint` file.**
Every `callback` in a `.slint` file has a comment explaining when it fires and what the Rust side is expected to do with it.

```slint
// Fired when the user presses Send or hits Enter in the text input.
// Rust: sends Command::SendDirectMessage to the backend worker.
callback send-message(string);
```

**SL-05. ListView is used for all scrollable message lists.**
Slint's `ListView` with a `StandardListView`-style model handles virtualization correctly for long chat histories. Do not use a `for` loop over a `VerticalBox` for message lists — it does not virtualize and will degrade with long histories.

**SL-06. Animations use Slint's built-in `animate` keyword.**
Do not use manual timer-based animation. Use `animate property { duration: 200ms; easing: ease-in-out; }` on state transitions.

---

## 9. UX Decisions & Rationale

### Why Slint over egui for this UX?
Slint's declarative `.slint` markup produces genuinely polished, consumer-grade interfaces with far less code than egui's immediate-mode API. Chat bubble layout, list scrolling, and state-driven animations are natural in Slint. The separation between `.slint` (what it looks like) and `src/ui/` (how it connects) also enforces clean separation of concerns — the UX designer and the Rust engineer can work in parallel without stepping on each other.

### Why no profile photos?
Profile photos require either: (a) a central server to host them, or (b) P2P file transfer on every contact add. Both are unnecessary complexity. Deterministic avatar colours (derived from NodeId hash) give visual identity without storage overhead.

### Why show the key fingerprint in the chat header persistently?
Users should be able to verify who they are talking to without hunting through menus. Showing the truncated fingerprint in the header — and making full verification one tap away — normalises security practices rather than hiding them.

### Why is the "queued" state visible and explicit?
A message that is silently stuck feels like a broken app. A message that clearly says `⏳ queued · will send when Eric is online` feels like a trustworthy system doing its job. The honesty builds confidence in the product.

### Why no camera for file attachments?
Camera integration requires platform permission handling and media APIs that add significant complexity. File picker (documents, images from gallery) achieves the same result with far less risk. Camera is a documented future extension.

### Why is the desktop left panel fixed-width?
A resizable panel adds drag-state, min/max constraints, and persistence complexity not worth the effort for a project demonstrating networking and cryptographic architecture. Fixed-width keeps the Slint layout predictable.

### Why no per-message delivery receipts for groups?
Gossip protocol is broadcast — there is no per-recipient ACK at the application level without implementing a complex separate ACK layer over gossip. For groups, "broadcast to swarm" is the meaningful delivery state. Showing `✓✓` that might mean "one person got it" would be dishonest.

---

*Last Updated: 2026-03-30*
*Project: NodeChat — Secure Decentralized Chat*
*Document: UX Flow & Interface Design v1.2 — Latest Crate Versions*
