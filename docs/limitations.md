# NodeChat Limitations

## Purpose

This document states the current limits of NodeChat as it exists today. Its role is to keep the project credible during defense, future documentation work, and later website development.

These limitations should not be presented as failures. They define the current maturity level and scope of the project.

## 1. The App Is A Strong Academic Prototype, Not A Finished Consumer Platform

NodeChat already demonstrates real application behavior, including local identity management, direct messaging, group messaging, local persistence, and visible message-state feedback.

However, it should still be presented as:

- a serious final-year project
- a working peer-to-peer messaging prototype
- a foundation for future improvement

It should not be presented as a fully matured commercial messaging service.

## 2. Verification Is Present, But Trust Workflow Is Still Basic

The app now correctly separates secure-session readiness from manual contact verification. That is an important strength.

At the same time, the current verification model is still simple:

- verification is a manual toggle in the app
- the app does not yet present a richer trust ceremony or advanced identity-checking workflow
- trust decisions remain user-controlled, but the process is intentionally lightweight

This should be described as a practical trust marker rather than a complete trust-management system.

## 3. Queued Delivery Has Practical Limits

NodeChat tracks queued messages and can retry pending work when communication resumes. This is useful and should be defended as real message-state handling.

Still, queued delivery should be described carefully:

- queued work depends on the app reconnecting and continuing pending communication
- it should not be described as a guaranteed always-available offline delivery system
- the current project scope does not position NodeChat as a full store-and-forward messaging network

This means the app demonstrates delivery-state awareness, but not every reliability guarantee expected from a large-scale commercial messenger.

## 4. Group Workflow Is Functional, But Still Narrow In Scope

Group messaging is implemented and usable, but the current model is intentionally focused.

Current limits include:

- group onboarding depends on invitation exchange through direct conversations
- group management is centered on creation, invitation, participation, and leaving
- advanced moderation, role management, and complex membership controls are not part of the current app scope

This should be presented as a clean and understandable group model, not as a feature-complete community platform.

## 5. Identity Recovery And Portability Are Limited

NodeChat gives each installation a local identity and supports local protection and full reset.

What the current app does not yet emphasize is a broad recovery or portability workflow. In practical terms:

- identity ownership is local to the app instance
- reset is supported, but rich recovery/export flows are not a major part of the current user experience
- the project should not imply a fully developed multi-device identity-management system

This is consistent with the project’s current scope and should be stated plainly.

## 6. The Project Is Not Yet A Full Multi-Platform Product Rollout

The codebase and app design point toward broader platform use, but the project should still be described in terms of its implemented demonstration scope rather than a finalized mass-deployment strategy.

That means NodeChat should not currently overclaim:

- large-scale deployment readiness
- polished production operations
- enterprise-grade administration or support workflows

## 7. Some Supporting Assets Are Still Catching Up To The App

The app itself has already been tightened in comments, wording, and behavior. The documentation and future public-facing website are now being rebuilt from that stronger app foundation.

This means:

- the app should remain the primary source of truth
- the docs should be written from current implementation
- the future `/site` should reuse those docs rather than introduce a different story

This is not an app weakness. It is a project-ordering decision: application first, narrative second.

## How To Present These Limitations

During defense or public presentation, the best framing is:

- NodeChat already demonstrates meaningful decentralized messaging behavior
- the project is honest about what is implemented and what remains outside current scope
- the documented limitations are part of a disciplined engineering approach, not uncertainty about the project

That framing keeps the project professional, logical, and credible.
