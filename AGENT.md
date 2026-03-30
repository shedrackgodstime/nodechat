# NodeChat — Agent Operational Rules

**Status:** Mandatory for all AI Agent interactions.

## 1. Core Reference Documents
The agent must always prioritize and follow the instructions in these three documents:
1. [ARCHITECTURE.md](file:///home/kristency/Projects/nodechat/ARCHITECTURE.md) — The system design, module hierarchy, and implementation phases.
2. [RULES.md](file:///home/kristency/Projects/nodechat/RULES.md) — Strict engineering, coding, and testing standards.
3. [UX_FLOW.md](file:///home/kristency/Projects/nodechat/UX_FLOW.md) — Interface design, interaction states, and user journey.

## 2. Code Generation & Implementation
- **No Guessing Code:** Never assume or guess how a library or function works. If you are unsure or haven't used a specific version before, verify it first.
- **Always Verify Crate Versions:** Always check `Cargo.toml` and documentation to ensure you are using the correct API for the specified version (e.g., `iroh 0.97.0`).
- **If Unsure, Indicate It:** If you face an error you cannot solve or encounter a concept you don't fully understand, explicitly state it. The user will provide documentation, sample code, or a fix.
- **No Placeholder Code:** Every line of code must be functional and follow the strict `RULES.md` (e.g., no `unwrap()`, proper error handling).

## 3. Communication
- If a proposed implementation deviates from `ARCHITECTURE.md`, you MUST flag it and ask for justification before proceeding.
- Before writing major modules, provide a brief summary of how it complies with the "Non-Negotiables" in `RULES.md`.