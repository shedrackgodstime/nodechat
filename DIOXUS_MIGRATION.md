# NodeChat — Dioxus Migration Guide

**Project:** NodeChat — Secure Decentralized Chat  
**Goal:** Migrate from egui to Dioxus for cross-platform UI  
**Covers:** Architecture · Setup · Components · State Management · Platform-Specific

---

## Table of Contents

1. [Why Dioxus?](#1-why-dioxus)
2. [Project Structure](#2-project-structure)
3. [Adding Dioxus Dependencies](#3-adding-dioxus-dependencies)
4. [Backend API Reference](#4-backend-api-reference)
5. [State Management](#5-state-management)
6. [Component Architecture](#6-component-architecture)
7. [Screen Implementation](#7-screen-implementation)
8. [Platform-Specific Considerations](#8-platform-specific-considerations)
9. [UX Reference (From UX_FLOW.md)](#9-ux-reference)
10. [Migration Checklist](#10-migration-checklist)

---

## 1. Why Dioxus?

| egui | Dioxus |
|------|--------|
| Immediate mode | Declarative components |
| Single `App::ui()` render | Composable components |
| `&mut self` state | Signals & hooks |
| No native mobile widgets | Full native styling |
| Limited ecosystem | React-like ecosystem |
| Hard to migrate | Easy incremental migration |

**Dioxus advantages for NodeChat:**
- Native mobile rendering on iOS/Android
- Familiar React-like patterns
- Better component reusability
- CSS-like styling (Dioxus Studio)
- Excellent Rust integration

---

## 2. Project Structure

After migration, the project structure should be:

```
nodechat/
├── Cargo.toml                 # Dioxus dependencies here
├── src/
│   ├── main.rs               # Entry point (desktop)
│   ├── lib.rs                # Library root
│   ├── core/                  # Business logic (unchanged)
│   │   ├── commands.rs       # Command enum
│   │   └── mod.rs            # NodeChatWorker
│   ├── crypto/                # Cryptography (unchanged)
│   ├── p2p/                   # Networking (unchanged)
│   ├── storage/               # SQLite (unchanged)
│   ├── api.rs                # Backend API (use this)
│   ├── api_types.rs          # Shared types (use this)
│   └── ui/                   # Dioxus UI
│       ├── main.rs           # Mobile entry
│       ├── app.rs            # Root component
│       ├── components/       # Reusable components
│       │   ├── message_bubble.rs
│       │   ├── contact_row.rs
│       │   ├── chat_input.rs
│       │   ├── bottom_bar.rs
│       │   └── ...
│       ├── screens/           # Screen components
│       │   ├── welcome.rs
│       │   ├── setup_name.rs
│       │   ├── identity_card.rs
│       │   ├── chat_list.rs
│       │   ├── chat_window.rs
│       │   ├── contacts.rs
│       │   └── settings.rs
│       └── state/             # Global state
│           └── app_state.rs
├── assets/                    # Static assets
└── styles/                   # CSS (optional)
```

---

## 3. Adding Dioxus Dependencies

Add to `Cargo.toml`:

```toml
[dependencies]
# Core Dioxus
dioxus = "0.15"
dioxus-router = "0.15"

# Platform-specific
[target.'cfg(not(target_os = "android"))'.dependencies]
dioxus-desktop = "0.15"
wry = "0.45"  # For window management

[target.'cfg(target_os = "android")'.dependencies]
dioxus-mobile = "0.15"
```

For full mobile support with navigation:

```toml
[dependencies]
dioxus = { version = "0.15", features = ["router", "signals"] }
dioxus-mobile = "0.15"  # For Android/iOS
```

---

## 4. Backend API Reference

### 4.1 Initialization

```rust
use nodechat::api::{init_backend, BackendHandles, Command, AppEvent};

fn main() {
    // Desktop
    dioxus::launchDesktop(|| Component::<App>::new());
    
    // Mobile  
    dioxus::mobile::launch_mobalie(|| Component::<App>::new());
}

// Initialize backend (do this early in app init)
let handles = init_backend("nodechat.sqlite".to_string())
    .await
    .expect("Failed to initialize backend");
```

### 4.2 Event Subscription

Subscribe to backend events for reactive updates:

```rust
use nodechat::api::AppEvent;

// In your root component's use_effect
let handles = state.clone();
use_effect(move || {
    let mut rx = handles.subscribe();
    
    dioxus::spawn(async move {
        while let Ok(event) = rx.recv().await {
            match event {
                AppEvent::IncomingMessage { sender, plaintext } => {
                    // Add to messages state
                }
                AppEvent::IdentityCreated { node_id, ticket, display_name } => {
                    // Update identity state
                }
                AppEvent::ErrorMessage(msg) => {
                    // Show error toast
                }
                _ => {}
            }
        }
    });
});
```

### 4.3 Available Commands

```rust
use nodechat::api::Command;

// Send direct message
handles.send_command(Command::SendDirectMessage {
    target: "nBq3...Kx7R".to_string(),
    plaintext: "Hello!".to_string(),
}).await;

// Add contact by ticket
handles.send_command(Command::AddContactByTicket {
    ticket: "iroh://...".to_string(),
}).await;

// Update profile
handles.send_command(Command::UpdateProfile {
    display_name: "My Name".to_string(),
}).await;

// Create group
handles.send_command(Command::CreateGroup {
    name: "Project Team".to_string(),
}).await;

// Send group message
handles.send_command(Command::SendGroupMessage {
    topic: topic_id,  // [u8; 32]
    plaintext: "Hello group!".to_string(),
}).await;

// Refresh identity
handles.send_command(Command::RefreshIdentity).await;
```

### 4.4 Available Events

```rust
use nodechat::api::AppEvent;

match event {
    // New direct message received
    AppEvent::IncomingMessage { sender, plaintext } => { }
    
    // Message delivery status update
    AppEvent::MessageStatusUpdate { id, status } => { }
    
    // Group message received
    AppEvent::IncomingGroupMessage { topic, sender, plaintext } => { }
    
    // Group invite received
    AppEvent::GroupInviteReceived { topic, group_name } => { }
    
    // Identity ready
    AppEvent::IdentityCreated { node_id, ticket, display_name } => { }
    
    // Identity generation failed
    AppEvent::IdentityGenerationFailed(msg) => { }
    
    // Error from backend
    AppEvent::ErrorMessage(msg) => { }
    
    // Backend is ready
    AppEvent::BackendReady => { }
}
```

---

## 5. State Management

### 5.1 Global App State

```rust
// src/ui/state/app_state.rs
use dioxus::prelude::*;
use nodechat::api::{BackendHandles, AppEvent};

#[derive(Clone, Signal, PartialEq)]
pub enum ViewState {
    Welcome,
    SetupName,
    Generating,
    IdentityCard,
    MainChat,
}

#[derive(Clone, Signal, PartialEq)]
pub enum MainTab {
    Chats,
    Contacts,
    Settings,
}

#[derive(Clone, Signal)]
pub struct AppState {
    pub view_state: ViewState,
    pub current_tab: MainTab,
    pub display_name: String,
    pub local_node_id: Option<String>,
    pub local_ticket: Option<String>,
    pub ticket_input: String,
    pub chat_input: String,
    pub contacts: Vec<String>,
    pub active_chat: Option<String>,
    pub messages: std::collections::HashMap<String, Vec<(String, String)>>,
    pub handles: BackendHandles,
}

impl AppState {
    pub fn new(handles: BackendHandles) -> Self {
        Self {
            view_state: ViewState::Welcome,
            current_tab: MainTab::Chats,
            display_name: String::new(),
            local_node_id: None,
            local_ticket: None,
            ticket_input: String::new(),
            chat_input: String::new(),
            contacts: Vec::new(),
            active_chat: None,
            messages: std::collections::HashMap::new(),
            handles,
        }
    }
}
```

### 5.2 Using State in Components

```rust
use crate::state::AppState;

fn ChatWindow(cx: Scope) -> Element {
    let state = use_context::<AppState>(cx).unwrap();
    
    let active_chat = state.active_chat();
    
    if let Some(peer_id) = active_chat {
        cx.render(rsx! {
            div {
                // Message list
                for (sender, text) in state.messages.get(peer_id).iter() {
                    MessageBubble {
                        sender: sender.clone(),
                        text: text.clone(),
                    }
                }
                
                // Input
                ChatInput {}
            }
        })
    } else {
        cx.render(rsx! {
            div { "Select a chat to start messaging" }
        })
    }
}
```

---

## 6. Component Architecture

### 6.1 Reusable Components

Create in `src/ui/components/`:

```
components/
├── message_bubble.rs    # Chat message bubble
├── contact_row.rs       # Contact list item
├── chat_input.rs       # Message input with send
├── bottom_bar.rs       # Mobile tab navigation
├── header.rs           # App header with status
├── avatar.rs           # User avatar (initials)
├── status_dot.rs       # Network status indicator
├── loading_spinner.rs  # Loading indicator
└── toast.rs           # Error/success notifications
```

### 6.2 Example: Message Bubble

```rust
// src/ui/components/message_bubble.rs
use dioxus::prelude::*;

#[props]
pub struct MessageBubbleProps {
    pub sender: String,
    pub text: String,
    pub timestamp: Option<String>,
    pub is_me: bool,
    pub status: Option<String>, // "sending", "sent", "delivered", "queued"
}

pub fn MessageBubble(cx: Scope<MessageBubbleProps>) -> Element {
    let is_me = cx.props.is_me;
    
    cx.render(rsx! {
        div {
            class: "message-row",
            justify_content: if is_me { "flex-end" } else { "flex-start" },
            
            div {
                class: "message-bubble",
                background: if is_me { "#1A5FA8" } else { "#2C2C2E" },
                
                div {
                    class: "message-text",
                    color: "white",
                    "{cx.props.text}"
                }
                
                if let Some(timestamp) = &cx.props.timestamp {
                    div {
                        class: "message-timestamp",
                        color: "#8E8E93",
                        "{timestamp}"
                    }
                }
                
                if let Some(status) = &cx.props.status {
                    div {
                        class: "message-status",
                        color: "#8E8E93",
                        "{status}"
                    }
                }
            }
        }
    })
}
```

### 6.3 Example: Contact Row

```rust
// src/ui/components/contact_row.rs
use dioxus::prelude::*;

#[props]
pub struct ContactRowProps {
    pub name: String,
    pub node_id: String,
    pub last_message: Option<String>,
    pub timestamp: Option<String>,
    pub is_verified: bool,
    pub is_online: bool,
    pub is_selected: bool,
    pub on_click: Option<Callback>,
}

pub fn ContactRow(cx: Scope<ContactRowProps>) -> Element {
    let initials = cx.props.name
        .split_whitespace()
        .map(|s| s.chars().next().unwrap_or('?'))
        .collect::<String>();
    
    cx.render(rsx! {
        div {
            class: "contact-row",
            background: if cx.props.is_selected { "#2C2C2E" } else { "transparent" },
            onClick: move |_| {
                if let Some(cb) = &cx.props.on_click {
                    cb.call(());
                }
            },
            
            // Avatar
            div {
                class: "avatar",
                background: hash_color(&cx.props.node_id),
                "{initials}"
            }
            
            // Info
            div {
                class: "contact-info",
                div {
                    class: "contact-name",
                    "{cx.props.name}"
                    if cx.props.is_verified {
                        span { "✓" }
                    }
                }
                if let Some(msg) = &cx.props.last_message {
                    div {
                        class: "last-message",
                        "{msg}"
                    }
                }
            }
            
            // Status dot
            StatusDot { online: cx.props.is_online }
        }
    })
}

fn hash_color(node_id: &str) -> String {
    // Deterministic color from node_id
    let hash = node_id.bytes().fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
    let hue = hash % 360;
    format!("hsl({}, 60%, 50%)", hue)
}
```

---

## 7. Screen Implementation

### 7.1 Screen Structure

```
screens/
├── welcome.rs          # Welcome screen (FL-01)
├── setup_name.rs       # Set display name (FL-02)
├── generating.rs      # Identity generation (FL-03)
├── identity_card.rs   # Show NodeId/QR (FL-04)
├── chat_list.rs       # Main chat list (MN-01)
├── chat_window.rs     # Active chat (CH-01)
├── contacts.rs        # Contacts list (MN-02)
├── settings.rs        # Settings (MN-03)
└── add_contact.rs     # Add contact flow (CO-01)
```

### 7.2 Welcome Screen (FL-01)

```rust
// src/ui/screens/welcome.rs
use dioxus::prelude::*;
use crate::state::AppState;

pub fn WelcomeScreen(cx: Scope) -> Element {
    let state = use_context::<AppState>(cx).unwrap();
    
    cx.render(rsx! {
        div {
            class: "welcome-screen",
            
            div { class: "logo", "NodeChat" }
            div { class: "tagline", "Secure. Private. Yours." }
            
            button {
                class: "primary-button",
                onClick: |_| {
                    state.view_state.set(ViewState::SetupName);
                },
                "Get Started"
            }
            
            div {
                class: "footer",
                "No account. No phone number. No server."
            }
        }
    })
}
```

### 7.3 Setup Name Screen (FL-02)

```rust
// src/ui/screens/setup_name.rs
use dioxus::prelude::*;
use crate::state::AppState;

pub fn SetupNameScreen(cx: Scope) -> Element {
    let state = use_context::<AppState>(cx).unwrap();
    let mut name = use_state(cx, || String::new());
    
    let is_valid = name.get().len() >= 1 && name.get().len() <= 32;
    
    cx.render(rsx! {
        div {
            class: "setup-screen",
            
            div { class: "title", "Set up your identity" }
            div { class: "subtitle", "What should people call you?" }
            
            input {
                r#type: "text",
                placeholder: "Display Name",
                value: "{name}",
                onInput: |e| name.set(e.value.to_string()),
            }
            
            button {
                class: "primary-button",
                disabled: !is_valid,
                onClick: |_| {
                    state.display_name.set(name.get().clone());
                    // Send command to backend
                    let handles = state.handles.clone();
                    dioxus::spawn(async move {
                        handles.send_command(
                            nodechat::api::Command::UpdateProfile { 
                                display_name: name.get().clone() 
                            }
                        ).await.ok();
                    });
                    state.view_state.set(ViewState::Generating);
                },
                "Continue"
            }
        }
    })
}
```

### 7.4 Chat Window (CH-01)

```rust
// src/ui/screens/chat_window.rs
use dioxus::prelude::*;
use crate::state::AppState;
use crate::components::{MessageBubble, ChatInput};

pub fn ChatWindowScreen(cx: Scope) -> Element {
    let state = use_context::<AppState>(cx).unwrap();
    
    let active_chat = state.active_chat();
    
    if let Some(peer_id) = active_chat {
        let messages = state.messages.get(peer_id).cloned().unwrap_or_default();
        
        cx.render(rsx! {
            div {
                class: "chat-window",
                
                // Header
                div {
                    class: "chat-header",
                    button {
                        onClick: |_| state.active_chat.set(None),
                        "←"
                    }
                    div { class: "peer-name", "{peer_id}" }
                }
                
                // Messages
                div {
                    class: "messages-list",
                    for (sender, text) in messages.iter() {
                        MessageBubble {
                            sender: sender.clone(),
                            text: text.clone(),
                            is_me: sender == "Me",
                        }
                    }
                }
                
                // Input
                ChatInput { peer_id: peer_id.clone() }
            }
        })
    } else {
        cx.render(rsx! {
            div {
                class: "empty-chat",
                "Select a conversation to start messaging."
            }
        })
    }
}
```

### 7.5 Mobile Main Layout

```rust
// src/ui/screens/mobile_main.rs
use dioxus::prelude::*;
use crate::state::{AppState, MainTab, ViewState};

pub fn MobileMainScreen(cx: Scope) -> Element {
    let state = use_context::<AppState>(cx).unwrap();
    
    let is_mobile = use_window_width(cx) < 600;
    
    if !is_mobile {
        return cx.render(rsx! { DesktopMainScreen {} });
    }
    
    // Check if in chat
    if state.active_chat().is_some() {
        return cx.render(rsx! { super::chat_window::ChatWindowScreen {} });
    }
    
    cx.render(rsx! {
        div {
            class: "mobile-layout",
            
            // Tab content
            match state.current_tab() {
                MainTab::Chats => rsx! { super::chat_list::ChatListScreen {} },
                MainTab::Contacts => rsx! { super::contacts::ContactsScreen {} },
                MainTab::Settings => rsx! { super::settings::SettingsScreen {} },
            }
            
            // Bottom bar
            super::components::BottomBar {}
        }
    })
}
```

---

## 8. Platform-Specific Considerations

### 8.1 Desktop Entry Point

```rust
// src/ui/main.rs (desktop)
use dioxus::prelude::*;

fn main() {
    dioxus::launchDesktop(App)
}

fn App(cx: Scope) -> Element {
    // Initialize backend
    let handles = /* get from state manager */;
    
    cx.render(rsx! {
        div {
            class: "desktop-layout",
            // Left panel - contacts
            // Right panel - chat
        }
    })
}
```

### 8.2 Mobile Entry Point

```rust
// src/ui/main.rs (mobile - in lib.rs)
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    // Initialize Android logging
    android_logger::init_once(/* config */);
    
    // Initialize backend
    let handles = /* get from state manager */;
    
    dioxus::mobile::launch(
        app,
        App,
        /* config */
    );
}
```

### 8.3 Keyboard Handling

Dioxus handles mobile keyboards well, but for chat input:

```rust
fn ChatInput(cx: Scope) -> Element {
    let state = use_context::<AppState>(cx).unwrap();
    let mut input = use_state(cx, || String::new());
    
    cx.render(rsx! {
        div {
            class: "chat-input-row",
            
            input {
                r#type: "text",
                placeholder: "Type a message...",
                value: "{input}",
                onInput: |e| input.set(e.value.to_string()),
                onSubmit: |_| {
                    if !input.get().is_empty() {
                        // Send message
                        state.chat_input.set(input.get().clone());
                        input.set(String::new());
                    }
                }
            }
            
            button {
                disabled: input.get().is_empty(),
                onClick: |_| {
                    // Send
                },
                "Send"
            }
        }
    })
}
```

### 8.4 Copy/Paste on Android

Dioxus on mobile uses native text inputs which should support copy/paste. If issues persist:

```rust
// Use native HTML input on mobile
input {
    r#type: "text",
    // This enables native keyboard and clipboard
}
```

---

## 9. UX Reference

For detailed UX specs (colors, layouts, flows), see `UX_FLOW.md`. Key points:

### Color Tokens

| Token | Dark Mode | Usage |
|-------|-----------|-------|
| `surface-primary` | `#1C1C1E` | Main backgrounds |
| `surface-secondary` | `#2C2C2E` | Sidebar, input areas |
| `accent` | `#4A9EE8` | Sent bubbles, buttons |
| `accent-success` | `#34C774` | Online indicators |
| `accent-warning` | `#F4A623` | Relay mode |
| `accent-danger` | `#FF453A` | Errors |
| `text-primary` | `#FFFFFF` | Main text |
| `text-secondary` | `#8E8E93` | Timestamps |

### Screen Flow

```
First Launch:
Welcome → SetupName → Generating → IdentityCard → MainChat

Returning User:
[PasswordGate?] → MainChat

MainChat:
ChatList ↔ Contacts ↔ Settings
    ↓
ChatWindow (1:1 or Group)
```

---

## 10. Migration Checklist

- [ ] Add Dioxus dependencies to Cargo.toml
- [ ] Create `src/ui/` directory structure
- [ ] Implement `AppState` with signals
- [ ] Wire up `init_backend()` at app start
- [ ] Set up event subscription loop
- [ ] Create reusable components (MessageBubble, ContactRow, etc.)
- [ ] Implement all screens from UX_FLOW.md
- [ ] Add desktop layout (two-panel)
- [ ] Add mobile layout (bottom tabs)
- [ ] Test copy/paste on Android
- [ ] Test keyboard handling
- [ ] Build and test on desktop
- [ ] Build and test on Android
- [ ] Remove old egui code
- [ ] Update README

---

## Quick Start Template

```rust
// Minimum viable Dioxus app for NodeChat

use dioxus::prelude::*;
use nodechat::api::{init_backend, Command, AppEvent};

fn main() {
    // Runtime
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    
    let handles = rt.block_on(async {
        init_backend("nodechat.sqlite".to_string()).await
    }).unwrap();
    
    // Launch UI
    dioxus::launch(move || {
        rsx! {
            div {
                "NodeChat"
                // Your UI here
            }
        }
    });
}
```

---

## Notes

- The backend (`core/`, `crypto/`, `p2p/`, `storage/`) **does not change**
- Only the UI layer (`src/ui/`) needs rewriting
- Use the API in `src/api.rs` — never directly call backend modules
- All types are in `src/api_types.rs` for framework-agnostic serialization
- Test on both platforms early — don't wait until "done"
