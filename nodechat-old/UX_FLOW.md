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
8. [UX Decisions & Rationale](#8-ux-decisions--rationale)

---

## 1. Design Philosophy

NodeChat's visual design follows three principles:

**1. Honest UI.** The interface never lies about delivery state, connection health, or encryption status. If a message is queued because a peer is offline, it says "queued." If the node is connected through a relay instead of directly, it says so. Users are treated as adults who can handle reality.

**2. Familiar shell, different guts.** The surface layout intentionally resembles WhatsApp/Telegram — left panel contacts, right panel messages, input bar at the bottom. The difference is what's *under* that shell: no phone number, no account, no central server. Familiarity lowers the learning curve. The cryptographic reality surfaces only when relevant.

**3. Mobile-first, desktop-enhanced.** The core chat experience is designed for mobile first. The desktop layout takes the same flows and adds a persistent two-panel or three-panel layout because screen space allows it — not because desktop gets different features.

### Colour Tokens (both platforms)

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

**What it shows:**
- App name and tagline
- Single primary CTA: "Get Started"
- One-liner below reassuring users nothing personal is collected

**What happens in the background:**
- Nothing yet. The keypair is NOT generated here. We wait until the user confirms their name so identity generation feels intentional.

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
│  ┌──────────────────────────┐   │
│  │  You can change this     │   │
│  │  anytime in Settings.    │   │
│  └──────────────────────────┘   │
│                                 │
│   ┌─────────────────────────┐   │
│   │       Continue          │   │
│   └─────────────────────────┘   │
│                                 │
└─────────────────────────────────┘
```

**Validation:**
- Name must be 1–32 characters
- No special characters that break display (limit to Unicode letters, numbers, spaces, basic punctuation)
- "Continue" button is disabled until valid input is provided

**Transitions to:** Screen 2.3

---

### Screen 2.3 — Generating Identity (Loading)

```
┌─────────────────────────────────┐
│                                 │
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
│                                 │
└─────────────────────────────────┘
```

**What happens here (backend):**
1. X25519/Ed25519 keypair is generated
2. Public key is derived → becomes the `NodeId`
3. Identity is written to local SQLite (encrypted if password was set)
4. Iroh endpoint binds and begins listening
5. Pkarr publishes the user's NodeAddr

**Duration:** Typically under 1 second on modern hardware. The screen is shown for a minimum of 1.5 seconds regardless — long enough for the user to read it, short enough to not feel like a problem.

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
│  want to chat with. They add    │
│  you using this ID.             │
│                                 │
│   ┌─────────────────────────┐   │
│   │      Go to Chats        │   │
│   └─────────────────────────┘   │
│                                 │
└─────────────────────────────────┘
```

**What it shows:**
- User's display name in a greeting
- QR code encoding their full NodeId (for easy in-person sharing)
- Truncated NodeId with a copy button
- Brief explanation of how contacts work
- "Go to Chats" CTA to enter the main app

**Key UX decision:** We show the NodeId immediately. The user hasn't added anyone yet, so teaching them how contact-sharing works here — when they're curious — is the right moment.

**Transitions to:** Screen 2.5 (optional password) OR directly to Main App

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

**Notes:**
- Skipping is explicitly allowed — not everyone needs this for a demo context
- If set, password is used to derive a key that encrypts the SQLite file and private key on disk
- This screen is shown once and can be configured again in Settings later
- Password is never transmitted anywhere

**Transitions to:** Main App (Contacts / Chat Home)

---

## 3. Returning User Flow (Both Platforms)

Every subsequent launch follows this path.

### Screen 3.1A — Password Gate (if password was set)

```
┌─────────────────────────────────┐
│                                 │
│                                 │
│           [App Icon]            │
│                                 │
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
│   Use biometrics instead  →     │  ← mobile only, if device supports it
│                                 │
└─────────────────────────────────┘
```

**Behaviour:**
- Wrong password: shake animation, "Incorrect password" below the field, clear the input
- 5 consecutive wrong attempts: 30-second cooldown with a visible countdown
- Correct password: SQLite decrypts, private key loads, proceed to main app

**What happens in the background on successful unlock:**
1. SQLite database decrypts
2. Private key loads into memory
3. Iroh endpoint binds
4. Pkarr re-publishes NodeAddr (in case IP changed since last session)
5. Backend worker starts flushing any queued messages immediately

### Screen 3.1B — Direct Launch (no password set)

No password screen. App loads directly. The same background steps fire, but faster since there's no decryption step.

### Screen 3.2 — Reconnecting State (Brief)

On both 3.1A and 3.1B, there is a brief moment (under 2 seconds typical) where the backend is bootstrapping. During this time:

- The main UI loads and renders immediately with cached state from SQLite
- A subtle status bar indicator shows "Connecting…"
- As each peer reconnects, their online indicator updates live
- Any queued messages begin flushing in the background without the user doing anything

The user does not see a loading screen. They see their chats immediately, with status indicators that update as connectivity is established.

---

## 4. Mobile Layout & Navigation

Mobile uses a **stack-based navigation model** — one screen at a time, with a back button. No persistent side panels. This matches what Android users already expect from every messaging app they've ever used.

### 4.1 Mobile Home Screen (Chat List)

```
┌───────────────────────────────────┐
│  NodeChat          [•] [+] [≡]   │  ← status dot, new chat, menu
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
│  │       ⏳ 2 msgs queued    │   │  ← offline queue indicator
│  └───────────────────────────┘    │
│                                   │
│  ┌───────────────────────────┐    │
│  │ [GR]  Project Group   Tue │   │
│  │       You: Let's ship it  │   │
│  └───────────────────────────┘    │
│                                   │
│  ┌───────────────────────────┐    │
│  │ [AV]  Anon 4f7c…     Mon │   │  ← unverified contact
│  │ ⚠️    Key not verified    │   │
│  └───────────────────────────┘    │
│                                   │
└───────────────────────────────────┘
│  [Chats]   [Contacts]  [Settings] │  ← bottom tab bar
└───────────────────────────────────┘
```

**Elements explained:**

`[•]` — Network status dot in the header. Tap to see detailed peer/relay info.
- Green: connected, direct P2P to most peers
- Amber: connected, but using relay for one or more peers
- Red: no network, fully offline

`[+]` — Start new chat. Opens Add Contact flow.

`[≡]` — App menu: Settings, Your NodeId, About.

**Chat row anatomy:**
- Avatar circle with initials (no profile photos for privacy — initials only)
- Display name + timestamp of last message
- Last message preview, OR status indicator if no message received yet
- `✓✓` = delivered, `✓` = sent, `⏳ 2 msgs queued` = offline
- Unread count badge on the right when messages are unread

**Tab bar (bottom):**
- **Chats** — conversation list (default tab)
- **Contacts** — phonebook of known NodeIds
- **Settings** — profile, password, NodeId, app preferences

---

### 4.2 Mobile Chat Screen (1:1)

```
┌───────────────────────────────────┐
│  ←  [AV]  Eric O.          [···] │
│       0xAF3B…C72E · direct · 🔒  │  ← key fingerprint, connection mode, lock
│───────────────────────────────────│
│                                   │
│          Tuesday, 29 Mar          │  ← date separator
│                                   │
│  ┌─────────────────────────┐      │
│  │ yo I pushed the ref doc │      │  ← incoming bubble (left)
│  │ to the repo             │      │
│  │                   12:38 │      │
│  └─────────────────────────┘      │
│                                   │
│      ┌─────────────────────────┐  │
│      │ received it, reviewing  │  │  ← outgoing bubble (right)
│      │ phase 2 now             │  │
│      │ 12:40              ✓✓   │  │
│      └─────────────────────────┘  │
│                                   │
│      ┌─────────────────────────┐  │
│      │ ⏱ disappears in 24h    │  │  ← ephemeral tag inside bubble
│      │ what do you think about │  │
│      │ adding supabase?        │  │
│      │ 12:41           routing…│  │  ← delivery state
│      └─────────────────────────┘  │
│                                   │
│  --- Key ratcheted · session #85  │  ← system event, muted
│                                   │
│───────────────────────────────────│
│  [📎]  Type a message…    [Send]  │
└───────────────────────────────────┘
```

**Header:**
- Back arrow → returns to chat list
- Avatar + display name (tappable → opens contact info sheet)
- Key fingerprint (truncated). Connection mode: `direct` or `via relay`
- Lock icon (🔒) → opens E2EE info sheet explaining the encryption in plain language
- `[···]` → chat options: view key, set ephemeral timer, clear history, verify contact

**Message bubbles:**
- Outgoing: right-aligned, `accent` colour
- Incoming: left-aligned, `surface-secondary` colour
- Timestamp inside the bubble, bottom-right
- Delivery states shown as text on outgoing messages:
  - `sending…` — being encrypted and dispatched
  - `routing…` — propagating through network
  - `✓` — sent from this device
  - `✓✓` — confirmed received by recipient node
  - `queued` — recipient offline, stored locally

**Ephemeral messages:**
- Tagged at the top of the bubble: `⏱ disappears in 24h`
- Timer is agreed when the user sets it in chat options
- Ephemeral messages show a visible countdown in their bubble when under 1 hour remaining

**System events (muted, centered):**
- `--- Session started · Forward Secrecy active ---`
- `--- Key ratcheted · session #85 ---`
- `--- Eric verified your key ---`

**Input bar:**
- `📎` → attachment: file picker only. No camera (for scope). Files are sent P2P.
- Text input expands vertically for long messages (max 4 lines before scrolling inside input)
- Send button becomes active when input is non-empty

---

### 4.3 Mobile Chat Screen (Group)

```
┌───────────────────────────────────┐
│  ←  [GR]  Project Group    [···] │
│       3 members · gossip swarm    │
│───────────────────────────────────│
│                                   │
│  ┌─────────────────────────┐      │
│  │ Eric O.                 │      │  ← sender name above bubble (group only)
│  │ pushed the build, check │      │
│  │ the repo                │      │
│  │                   15:02 │      │
│  └─────────────────────────┘      │
│                                   │
│  ┌─────────────────────────┐      │
│  │ Shedrack G.             │      │
│  │ on it, building now     │      │
│  │                   15:04 │      │
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

**Differences from 1:1:**
- Sender name shown above each incoming bubble (since multiple senders)
- Header shows member count and transport mode (`gossip swarm`)
- No per-message delivery receipts for group (gossip is broadcast — no per-recipient ACK)
- `[···]` → Group options: member list, your key in this group, leave group, invite member

---

### 4.4 Mobile Add Contact Flow

Triggered by tapping `[+]` on the chat list.

```
┌───────────────────────────────────┐
│  ←   Add Contact                  │
│───────────────────────────────────│
│                                   │
│   How do you want to add them?    │
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

**QR scan path:** Opens camera, scans NodeId QR, previews "Add Shedrack G.?" → confirm → added to local phonebook, connection attempt begins.

**Manual Node ID path:**

```
┌───────────────────────────────────┐
│  ←   Enter Node ID                │
│───────────────────────────────────│
│                                   │
│  Paste their Node ID below:       │
│                                   │
│  ┌───────────────────────────┐    │
│  │  nBq3...Kx7R         [📋] │    │
│  └───────────────────────────┘    │
│                                   │
│  Give them a name (local only):   │
│  ┌───────────────────────────┐    │
│  │  e.g. Eric               │    │
│  └───────────────────────────┘    │
│                                   │
│  ┌───────────────────────────┐    │
│  │       Add Contact         │    │
│  └───────────────────────────┘    │
│                                   │
└───────────────────────────────────┘
```

After adding: a connection attempt fires immediately. If the peer is online, a handshake completes and they appear with a green online indicator. If offline, they appear in the contact list grayed out with "Offline" until their node is discovered via Pkarr.

---

### 4.5 Mobile Key Verification Screen

Accessible from chat `[···]` → "Verify contact's key"

```
┌───────────────────────────────────┐
│  ←   Verify Eric O.               │
│───────────────────────────────────│
│                                   │
│  Compare these numbers with Eric  │
│  in person or via another channel.│
│                                   │
│  If they match, your conversation │
│  is secure and unmodified.        │
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
│  Someone may be intercepting this │
│  conversation. Do not continue.   │
│                                   │
└───────────────────────────────────┘
```

Once marked verified:
- Contact gets a green `Verified` badge in chat header and contact list
- System message appears in the chat: `--- You verified Eric's identity ---`

---

### 4.6 Mobile Settings Screen

```
┌───────────────────────────────────┐
│  ←   Settings                     │
│───────────────────────────────────│
│                                   │
│  IDENTITY                         │
│  ┌───────────────────────────┐    │
│  │  Display Name: Shedrack G.│    │
│  │  Node ID: nBq3…Kx7R  [↗] │    │
│  │  Show QR Code             │    │
│  └───────────────────────────┘    │
│                                   │
│  SECURITY                         │
│  ┌───────────────────────────┐    │
│  │  App Password         [>] │    │
│  │  Biometric Unlock     [>] │    │
│  └───────────────────────────┘    │
│                                   │
│  PRIVACY                          │
│  ┌───────────────────────────┐    │
│  │  Default ephemeral timer  │    │
│  │  [Off ▾]                  │    │
│  │  Read receipts    [ON  ●] │    │
│  └───────────────────────────┘    │
│                                   │
│  NETWORK                          │
│  ┌───────────────────────────┐    │
│  │  Connection status        │    │
│  │  8 peers · 1 via relay    │    │
│  │  Node address: [View]     │    │
│  └───────────────────────────┘    │
│                                   │
│  DANGER ZONE                      │
│  ┌───────────────────────────┐    │
│  │  Clear all messages   [>] │    │
│  │  Delete identity      [>] │    │
│  └───────────────────────────┘    │
│                                   │
└───────────────────────────────────┘
```

---

## 5. Desktop Layout & Navigation

Desktop uses a **persistent multi-panel layout**. The user can see contacts AND active chat simultaneously because the screen has the space for it. There is no navigation stack — panels update in place.

### 5.1 Desktop Layout Overview (Two-Panel)

```
┌──────────────────────────────────────────────────────────────────┐
│  NodeChat    [•] Connected · 8 peers · 1 relay      [≡] Settings │
├──────────────┬───────────────────────────────────────────────────┤
│  CONTACTS    │                                                   │
│  ─────────── │                                                   │
│  🔍 Search…  │                                                   │
│              │            Select a chat                          │
│  Eric O.     │            to start messaging.                    │
│  ✓ Verified  │                                                   │
│  12:41 ✓✓   │                                                   │
│              │                                                   │
│  Mama        │                                                   │
│  ⏳ Queued   │                                                   │
│  09:12       │                                                   │
│              │                                                   │
│  [Group]     │                                                   │
│  Project Grp │                                                   │
│  15:04       │                                                   │
│              │                                                   │
│  ──────────  │                                                   │
│  [+] New     │                                                   │
│  [⊕] Group   │                                                   │
└──────────────┴───────────────────────────────────────────────────┘
```

**Left panel (fixed, ~260px):**
- App title and global network status in the top bar
- Search input at the top
- Scrollable contact list: 1:1 contacts and groups mixed, sorted by last activity
- `[+] New chat` and `[⊕] New group` at the bottom
- `[≡] Settings` in the top-right corner of the bar

**Right panel (fills remaining width):**
- Empty state with prompt when no chat is selected
- Active chat when a contact is selected

---

### 5.2 Desktop Active Chat (1:1)

```
┌──────────────────────────────────────────────────────────────────┐
│  NodeChat    [•] Connected · 8 peers · 1 relay      [≡] Settings │
├──────────────┬───────────────────────────────────────────────────┤
│  CONTACTS    │  [AV] Eric O.                 [Verify] [···]      │
│  ─────────── │  0xAF3B…C72E · direct · 🔒                        │
│  🔍 Search…  │  ─────────────────────────────────────────────────│
│              │                                                   │
│  ► Eric O.   │           Tuesday, 29 Mar                         │
│  ✓ 12:41 ✓✓ │                                                   │
│              │  ┌────────────────────────────┐                   │
│  Mama        │  │ yo I pushed the ref doc    │                   │
│  ⏳ Queued   │  │ to the repo                │                   │
│  09:12       │  │                      12:38 │                   │
│              │  └────────────────────────────┘                   │
│  Project Grp │                                                   │
│  15:04       │          ┌────────────────────────────┐           │
│              │          │ received it, reviewing     │           │
│              │          │ phase 2 now                │           │
│              │          │ 12:40                  ✓✓ │           │
│              │          └────────────────────────────┘           │
│              │                                                   │
│              │  --- Key ratcheted · session #85 ---              │
│              │                                                   │
│              │  ─────────────────────────────────────────────────│
│  [+] New     │  [📎]  Type a message…                    [Send]  │
│  [⊕] Group   │                                                   │
└──────────────┴───────────────────────────────────────────────────┘
```

**Desktop-specific additions:**
- `[Verify]` button is always visible in the chat header (not buried in a menu) because screen space allows it
- Right panel header shows full fingerprint and connection mode persistently
- The left panel highlights the active chat with a subtle background accent
- Keyboard shortcut: `Enter` sends, `Shift+Enter` for newlines
- `Ctrl+K` opens search (focuses the search box in the left panel)

---

### 5.3 Desktop Settings (Right-Panel Modal)

On desktop, settings open as a right-side panel replacing the chat panel rather than a separate screen. This keeps the contact list visible.

```
┌──────────────────────────────────────────────────────────────────┐
│  NodeChat    [•] Connected · 8 peers · 1 relay      [≡] Settings │
├──────────────┬───────────────────────────────────────────────────┤
│  CONTACTS    │  Settings                              [✕ Close]  │
│  ─────────── │  ─────────────────────────────────────────────────│
│  🔍 Search…  │                                                   │
│              │  IDENTITY                                         │
│  Eric O.     │  ┌───────────────────────────────────────┐        │
│  Mama        │  │  Display Name    Shedrack G.    [Edit] │        │
│  Project Grp │  │  Node ID         nBq3…Kx7R      [Copy] │       │
│              │  │                                [Show QR]│       │
│              │  └───────────────────────────────────────┘        │
│              │                                                   │
│              │  SECURITY                                         │
│              │  ┌───────────────────────────────────────┐        │
│              │  │  App Password                  [Change]│        │
│              │  └───────────────────────────────────────┘        │
│              │                                                   │
│              │  PRIVACY                                          │
│              │  ┌───────────────────────────────────────┐        │
│              │  │  Default ephemeral timer      [Off  ▾] │       │
│              │  │  Read receipts              [●──── ON] │       │
│              │  └───────────────────────────────────────┘        │
│              │                                                   │
│              │  NETWORK STATUS                                   │
│              │  ┌───────────────────────────────────────┐        │
│              │  │  Peers: 8 direct · 1 via relay        │        │
│              │  │  Health: ████████░░ 80%               │        │
│              │  │  Your node address: [View full]       │        │
│              │  └───────────────────────────────────────┘        │
│              │                                                   │
│  [+] New     │  ─────────────────────────────────────────────────│
│  [⊕] Group   │  DANGER ZONE   [Clear messages] [Delete identity] │
└──────────────┴───────────────────────────────────────────────────┘
```

---

### 5.4 Desktop Add Contact / New Group

Also opens as a right-panel overlay, not a modal popup that blocks the whole window.

```
┌──────────────────────────────────────────────────────────────────┐
│  NodeChat    [•] Connected                          [≡] Settings │
├──────────────┬───────────────────────────────────────────────────┤
│  CONTACTS    │  Add Contact                           [✕ Close]  │
│  ─────────── │  ─────────────────────────────────────────────────│
│  🔍 Search…  │                                                   │
│              │  Paste their Node ID:                             │
│  Eric O.     │  ┌──────────────────────────────────────────┐     │
│  Mama        │  │  nBq3...Kx7R                        [📋] │     │
│  Project Grp │  └──────────────────────────────────────────┘     │
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
│              │                                                   │
│  [+] New     │                                                   │
│  [⊕] Group   │                                                   │
└──────────────┴───────────────────────────────────────────────────┘
```

---

## 6. Screen-by-Screen Reference

A complete map of every screen in the application.

### First Launch Screens (run once)
| ID | Screen | Trigger |
|---|---|---|
| FL-01 | Welcome | App first install |
| FL-02 | Set Display Name | After FL-01 |
| FL-03 | Generating Identity | After FL-02 confirmed |
| FL-04 | Your NodeId / QR | After keypair generated |
| FL-05 | Set Local Password (optional) | After FL-04 |

### Returning User Screens
| ID | Screen | Trigger |
|---|---|---|
| RU-01 | Password Gate | App launch, if password set |
| RU-01B | Direct Load | App launch, no password |

### Main Navigation
| ID | Screen | Platform |
|---|---|---|
| MN-01 | Chat List | Mobile (home tab) |
| MN-02 | Contacts List | Mobile (contacts tab) / Desktop (left panel) |
| MN-03 | Settings | Mobile (settings tab) / Desktop (right panel) |

### Chat Screens
| ID | Screen | Trigger |
|---|---|---|
| CH-01 | 1:1 Chat | Tap a direct contact |
| CH-02 | Group Chat | Tap a group |
| CH-03 | Chat Options Sheet | Tap `[···]` in any chat |
| CH-04 | Key Verification | Chat options → Verify |
| CH-05 | File Transfer | Attach and send a file |
| CH-06 | Ephemeral Timer Config | Chat options → Set timer |

### Contact Screens
| ID | Screen | Trigger |
|---|---|---|
| CO-01 | Add Contact | Tap `[+]` |
| CO-02 | Contact Info Sheet | Tap contact avatar in chat |
| CO-03 | New Group | Tap `[⊕]` |
| CO-04 | Group Info / Members | Chat options → Members |
| CO-05 | Invite to Group | Group info → Invite |

### System Screens
| ID | Screen | Trigger |
|---|---|---|
| SY-01 | Network Status Detail | Tap the status dot |
| SY-02 | E2EE Info Sheet | Tap the lock icon in chat |
| SY-03 | Danger Zone Confirm | Settings → Delete identity |

---

## 7. Shared UI Components

These components appear across both mobile and desktop and must behave identically.

### 7.1 Message Bubble

| State | Visual |
|---|---|
| Sending | Bubble renders immediately, faded slightly, `sending…` in place of timestamp |
| Routing | Full opacity, `routing…` italic beside timestamp |
| Sent | Single tick `✓` |
| Delivered | Double tick `✓✓` |
| Queued | `⏳ queued` shown as subtle tag, amber colour |
| Ephemeral | Timer tag at top of bubble; turns red when < 1 hour remaining |

### 7.2 Contact Row

Always shows:
- Avatar (initials, coloured deterministically from NodeId hash — same contact always has same colour)
- Display name
- Verification badge: `✓ Verified` (green) or `⚠ Unverified` (amber) or nothing
- Last message preview or queue status
- Timestamp of last activity
- Unread count badge (if applicable)
- Online indicator dot: green (direct), amber (relay), gray (offline)

### 7.3 Network Status Dot

Three states only:
- 🟢 Green: at least one direct peer connected
- 🟡 Amber: connected, but all peers via relay (reduced privacy)
- 🔴 Red: no network, all messages will queue

Tapping the dot on either platform opens a detail sheet showing:
```
Network Status
───────────────
Direct peers:     7
Relay peers:      1  (reduced privacy)
Queued messages:  2
Node uptime:      4m 32s
Your address:     [View full NodeAddr]
```

### 7.4 System Event Row (In Chat)

```
─────  Session started · Forward Secrecy active  ─────
```

- Centered, muted text colour
- No avatar, no timestamp, no interaction
- Used for: session start, key ratchet events, key verification, member join/leave (groups)

### 7.5 E2EE Info Sheet

Appears when user taps the 🔒 lock icon in any chat header.

```
┌──────────────────────────────────┐
│  🔒  End-to-End Encrypted         │
│──────────────────────────────────│
│                                  │
│  Messages in this chat are       │
│  encrypted before leaving your   │
│  device. Only you and Eric can   │
│  read them.                      │
│                                  │
│  Encryption:  ChaCha20-Poly1305  │
│  Key exchange: X25519 DH         │
│  Forward secrecy: Hash ratchet   │
│  Connection:  Direct P2P         │
│                                  │
│  Eric's key fingerprint:         │
│  0xAF3B C72E 991D 04F2 ...       │
│                                  │
│  [ Verify key with Eric → ]      │
│                                  │
└──────────────────────────────────┘
```

---

## 8. UX Decisions & Rationale

### Why no profile photos?
Profile photos require either: (a) a central server to host them, or (b) P2P file transfer on every contact add. Both are complexity the project doesn't need. Deterministic avatar colours (derived from the NodeId hash) give visual identity without storage overhead.

### Why show the key fingerprint in the chat header?
Users should be able to verify they are talking to who they think they are without hunting through menus. Showing the truncated fingerprint persistently — and making full verification one tap away — normalises security practices rather than hiding them.

### Why the "queued" state is visible and explicit?
A message that is silently "stuck" feels like a broken app. A message that clearly says `⏳ queued · will send when Eric is online` feels like a trustworthy system doing its job. The honesty builds confidence in the product rather than eroding it.

### Why no camera for file attachments on mobile (in scope)?
The camera integration requires `cargo-apk` permission handling and Android media APIs that add significant complexity. File picker (documents, images from gallery) achieves the same result with far less risk. Camera is a documented future extension.

### Why immediate-mode UI doesn't show loading spinners everywhere?
egui's immediate-mode model means the UI is always rendering from local state. When the backend fetches something, the cached state is shown immediately and updates when the event arrives — this is inherently more responsive than a loading spinner pattern. The only spinners used are for first-launch identity generation and the password gate, both of which are genuine blocking operations.

### Why the desktop left panel is fixed-width not resizable?
A resizable panel adds implementation complexity (drag state, min/max constraints, persistence) that is not worth the effort for a project that is primarily demonstrating networking and cryptographic architecture. Fixed-width keeps the egui layout code simple and predictable.

### Why groups have no per-message delivery receipts?
Gossip protocol is broadcast — there is no per-recipient acknowledgment at the application level without implementing a complex ACK layer on top of gossip. Single-chat delivery receipts work because there is one recipient. For groups, "sent to swarm" is the meaningful state. Showing false `✓✓` marks that might mean "one person got it" would be dishonest.

---

*Last Updated: 2026-03-30*  
*Project: NodeChat — Secure Decentralized Chat*  
*Document: UX Flow & Interface Design v1.0*
