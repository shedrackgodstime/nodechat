# NodeChat Website — Project Plan

## Overview
A showcase/documentation site for NodeChat, built with Qwik + Tailwind CSS, deployed on Cloudflare Pages.

## Design System

### Color Palette (from Slint theme)
| Token | Hex | Usage |
|-------|-----|-------|
| `accent` | `#4A9EE8` | Primary actions, CTAs |
| `surface-primary` | `#1C1C1E` | Main backgrounds |
| `surface-secondary` | `#2C2C2E` | Cards, sections |
| `surface-tertiary` | `#3A3A3C` | Borders, dividers |
| `accent-success` | `#34C774` | Online, verified states |
| `accent-warning` | `#F4A623` | Relay, unverified |
| `accent-danger` | `#FF453A` | Errors, destructive |
| `text-primary` | `#FFFFFF` | Main text |
| `text-secondary` | `#8E8E93` | Muted text |
| `text-tertiary` | `#636366` | Disabled, hints |

### Typography
- Primary font: Inter (already in assets/)
- Fallback: system-ui

## Site Structure

```
/site/src/routes/
├── index.tsx          # Landing page (Hero, Features, Team, CTA, Footer)
├── docs/
│   └── [...slug]/    # Dynamic docs from /docs folder
└── layout.tsx        # Shared layout (nav)
```

## Pages & Sections

### 1. Landing Page (index.tsx)
- **Hero Section**
  - App name + tagline
  - "Download" button (links to GitHub releases or shows "Coming Soon")
  - "Try it out" CTA
- **Features Section**
  - P2P Decentralized
  - End-to-End Encryption
  - Actor Model Architecture
  - Group Chat Support
- **Team Section** (anchor: `#team`)
  - Team members inline on main page
- **Tech Stack Section**
  - Rust + Slint + Iroh
- **Footer** (Copyright, Links, "Made with Slint")

### 2. Docs (/docs)
- Dynamically load MD files from `/docs` folder
- Use Qwik City MDX routes
- Sidebar navigation
- Note: `iroh_docs/` and `slint-ui_docs/` are gitignored, not included

### 3. Shared Components
- Navbar (Logo, Links: Home, Docs) — scroll to section via anchor

## Technical Notes

- **Tailwind CSS v4** already configured in package.json
- **Qwik City** for routing and MDX support
- **Cloudflare Pages** adapter ready (see deploy script)
- **Responsive**: Mobile-first, desktop optimized

## Execution Order

1. [ ] Set up Tailwind with custom theme colors
2. [ ] Create shared layout (navbar + footer)
3. [ ] Build Landing page (hero + features)
4. [ ] Build Team page
5. [ ] Set up Docs with dynamic MD loading
6. [ ] Add download button to hero
7. [ ] Test and verify build

## Status
- [x] Set up Tailwind with custom theme colors
- [x] Create shared layout (navbar + footer)
- [x] Build Landing page (hero + features + team + CTA + footer)
- [x] Set up Docs with dynamic MD loading from GitHub
- [x] Add download button to hero
- [x] Test and verify build

---

//we definately gonna add feed back and suggestions section.... and also if want to contribute....

*Last Updated: 2026-04-02*