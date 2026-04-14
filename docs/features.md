# NodeChat Features

## Feature Summary

NodeChat is centered on a small set of practical messaging features that already exist in the application. These features should be described in product terms first, with technical detail left for later reference documents.

## Local Identity

Each user works with a local identity created inside the app. This identity gives the user a recognizable profile inside NodeChat and provides the information needed to connect with other peers.

Key points:

- identity setup happens inside the app
- the app can be protected with a PIN-based unlock flow
- each user can share a connection ticket so other peers can add them

## Direct Peer Messaging

NodeChat supports direct conversations between two peers. A user can add another participant using a connection ticket or node identifier and then open a one-to-one conversation.

Direct messaging includes:

- saved contacts list
- direct conversation history
- visible peer connection state
- encrypted transport once a secure session is established

## Group Conversations

NodeChat also supports group messaging. A user can create a group, select members, and send invitations through direct communication channels already known to the app.

Group messaging includes:

- group creation with name and description
- saved group conversations
- invitation-based member onboarding
- shared group conversation view inside the main app

The app presents groups as peer-to-peer group conversations rather than as centrally hosted chat rooms.

## Secure Session Handling

The app distinguishes between transport readiness and user trust.

In practice, this means:

- a handshake establishes the secure session needed for communication
- verification is a separate manual action used to mark a contact as trusted
- the app does not treat those two ideas as the same thing

This is an important feature of the current app because it makes the trust model easier to explain and defend.

## Message State Feedback

NodeChat exposes delivery-related message states in the interface so users can understand what is happening to a message after sending it.

The current app uses message states such as:

- queued
- sent
- delivered
- read

This helps both usability and demonstration. It shows that the app is not only sending messages, but also tracking communication progress in a structured way.

## Local Persistence

NodeChat stores its working data locally so the app can restore state between sessions.

The local app data includes:

- identity details
- contacts
- group records
- message history

This local persistence is part of what makes the app feel like a complete application rather than a temporary networking demo.

## Status And User Feedback

The app includes visible feedback for important operations and state changes.

This includes:

- connection-stage labels
- online and offline indicators
- secure-session readiness cues
- in-app notices for success and error events
- confirmation flows for destructive actions

These features matter because they make the app easier to understand during normal use and easier to defend during project review.

## Current Feature Positioning

The current feature set should be presented as a strong academic prototype with real user-facing behavior. It already demonstrates identity handling, peer communication, group messaging, stateful UI feedback, and trust-aware interaction design.

It should not yet be described as a full production messaging platform with every advanced workflow completed.
