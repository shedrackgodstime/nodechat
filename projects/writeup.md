# DESIGN AND IMPLEMENTATION OF A SECURE DECENTRALIZED MESSAGING APPLICATION

**BY**

**SHEDRACK OJEISA GODSTIME**  
**REG. NO: ICT/2252410413**

**AND**

**IFEDAYO TOLUWALOPE MOYINOLUWA**  
**REG. NO: ICT/2252403308**

**DEPARTMENT OF COMPUTER SCIENCE**  
**OPTION: CYBER SECURITY**  
**[NAME OF INSTITUTION]**

**A PROJECT SUBMITTED TO THE DEPARTMENT OF COMPUTER SCIENCE IN PARTIAL FULFILLMENT OF THE REQUIREMENTS FOR THE AWARD OF HIGHER NATIONAL DIPLOMA (HND) IN COMPUTER SCIENCE**

**[MONTH, YEAR]**

---

## DECLARATION

We hereby declare that this project titled **Design and Implementation of a Secure Decentralized Messaging Application** was carried out by us and has not been submitted in part or in full to any other institution for the award of any degree or diploma. All sources consulted have been duly acknowledged.

**______________________________**  
Shedrack Ojeisa Godstime

**______________________________**  
Ifedayo Toluwalope Moyinoluwa

**Date: __________________**

## CERTIFICATION

This is to certify that this project titled **Design and Implementation of a Secure Decentralized Messaging Application** was carried out by **Shedrack Ojeisa Godstime** and **Ifedayo Toluwalope Moyinoluwa** under the supervision of **[Supervisor Name]** and has been approved as meeting the requirement for the award of Higher National Diploma (HND) in Computer Science.

**______________________________**  
Project Supervisor

**______________________________**  
Head of Department

**______________________________**  
External Examiner

**Date: __________________**

## DEDICATION

This work is dedicated to God Almighty and to our parents, guardians, lecturers, and everyone whose support, guidance, and sacrifice contributed to the successful completion of this project.

## ACKNOWLEDGEMENTS

We give thanks to God Almighty for life, strength, wisdom, and the grace to complete this project successfully. We sincerely appreciate our supervisor, **[Supervisor Name]**, for guidance, corrections, encouragement, and constructive advice throughout the course of this work.

We also acknowledge the Head of Department, members of staff of the Department of Computer Science, our colleagues, and friends whose suggestions and support were valuable during the planning, development, and documentation of this project. Finally, we appreciate our families for their patience, prayers, and continuous support.

## ABSTRACT

Instant messaging has become one of the most common forms of digital communication, yet many widely used messaging platforms are built around centralized infrastructure that introduces privacy risks, single points of failure, and strong dependence on service providers. This project presents the design and implementation of a secure decentralized messaging application named **NodeChat**, developed as a peer-to-peer messaging system that reduces reliance on central servers while preserving practical communication features.

The system was implemented using the Rust programming language, the Slint user interface framework, the Iroh peer-to-peer networking stack, and SQLite for local persistence. Security was addressed through local identity ownership, X25519-based shared secret derivation, SHA-256-based key derivation, and ChaCha20-Poly1305 authenticated encryption for message payload protection. The application supports local identity setup, optional PIN protection, direct peer messaging, invitation-based group messaging, message-state tracking, and local storage of conversations and contacts.

The result is a working academic prototype that demonstrates how decentralized messaging can be structured as a real software system rather than as a networking experiment alone. The study shows that peer-to-peer communication, local data ownership, and security-aware interaction design can be combined into a coherent application suitable for academic defense and future improvement.

## TABLE OF CONTENTS

- [CHAPTER ONE: INTRODUCTION](#chapter-one-introduction)
  - [1.1 Background of the Study](#11-background-of-the-study)
  - [1.2 Statement of the Problem](#12-statement-of-the-problem)
  - [1.3 Aim and Objectives of the Study](#13-aim-and-objectives-of-the-study)
  - [1.4 Significance of the Study](#14-significance-of-the-study)
  - [1.5 Scope and Delimitation of the Study](#15-scope-and-delimitation-of-the-study)
  - [1.6 Limitations of the Study](#16-limitations-of-the-study)
  - [1.7 Organisation of the Work](#17-organisation-of-the-work)
  - [1.8 Definition of Terms](#18-definition-of-terms)
- [CHAPTER TWO: LITERATURE REVIEW](#chapter-two-literature-review)
  - [2.0 Introduction](#20-introduction)
  - [2.1 Communication Network Models](#21-communication-network-models)
  - [2.2 Peer-to-Peer Networking and NAT Traversal](#22-peer-to-peer-networking-and-nat-traversal)
  - [2.3 Instant Messaging Protocols](#23-instant-messaging-protocols)
  - [2.4 Cryptography in Messaging Applications](#24-cryptography-in-messaging-applications)
  - [2.5 End-to-End Encryption in Practice](#25-end-to-end-encryption-in-practice)
  - [2.6 Local Data Storage and Persistence](#26-local-data-storage-and-persistence)
  - [2.7 Summary of Literature Review](#27-summary-of-literature-review)
- [CHAPTER THREE: SYSTEM ANALYSIS AND DESIGN](#chapter-three-system-analysis-and-design)
  - [3.0 Introduction](#30-introduction)
  - [3.1 Analysis of the Existing System](#31-analysis-of-the-existing-system)
  - [3.2 Analysis of the Proposed System](#32-analysis-of-the-proposed-system)
  - [3.3 Methodology](#33-methodology)
  - [3.4 Functional Requirements](#34-functional-requirements)
  - [3.5 Non-Functional Requirements](#35-non-functional-requirements)
  - [3.6 Architectural Design of the Proposed System](#36-architectural-design-of-the-proposed-system)
  - [3.7 Database Design](#37-database-design)
  - [3.8 Security Design](#38-security-design)
  - [3.9 User Interface Design](#39-user-interface-design)
  - [3.10 Chapter Summary](#310-chapter-summary)
- [CHAPTER FOUR: SYSTEM IMPLEMENTATION AND EVALUATION](#chapter-four-system-implementation-and-evaluation)
  - [4.0 Introduction](#40-introduction)
  - [4.1 Development Tools and Environment](#41-development-tools-and-environment)
  - [4.2 Implementation Overview](#42-implementation-overview)
  - [4.3 Implemented Features](#43-implemented-features)
  - [4.4 Testing and Evaluation](#44-testing-and-evaluation)
  - [4.5 Discussion of Results](#45-discussion-of-results)
  - [4.6 Chapter Summary](#46-chapter-summary)
- [CHAPTER FIVE: SUMMARY, CONCLUSION AND RECOMMENDATIONS](#chapter-five-summary-conclusion-and-recommendations)
  - [5.0 Introduction](#50-introduction)
  - [5.1 Summary](#51-summary)
  - [5.2 Conclusion](#52-conclusion)
  - [5.3 Recommendations](#53-recommendations)
  - [5.4 Suggestions for Further Work](#54-suggestions-for-further-work)
- [REFERENCES](#references)

---

## CHAPTER ONE: INTRODUCTION

### 1.1 Background of the Study

The development of modern communication systems has passed through several important stages, beginning from the telegraph and telephone, then advancing through radio communication, computer networking, and finally the global internet. With the expansion of the internet in the late twentieth century, electronic communication became more immediate, interactive, and widely accessible. Electronic mail, Internet Relay Chat (IRC), and early instant messaging systems such as ICQ, AOL Instant Messenger, and MSN Messenger demonstrated the growing need for real-time digital communication.

In the present era, messaging applications such as WhatsApp, Telegram, Signal, and Facebook Messenger have become central to both personal and professional interaction. These systems have transformed communication by allowing users to exchange text, media, and files almost instantly across long distances. However, most of these applications depend heavily on centralized client-server infrastructure, where service providers control account systems, routing, storage, and, in many cases, metadata.

Although centralized messaging systems are convenient and scalable, they introduce several important security and reliability concerns. A central server can become a single point of failure, a point of censorship, or a target for data compromise. Even where message content is encrypted, users often remain dependent on a provider for identity management, message transport, or availability. This creates a situation where communication privacy depends not only on cryptography, but also on institutional trust in the service operator.

Peer-to-peer (P2P) communication offers an alternative approach. In a P2P design, communicating devices connect more directly, reducing dependence on centralized control. When this model is combined with modern encryption techniques and local data ownership, it creates an opportunity to build a messaging application that better protects privacy, reduces central points of failure, and gives users more control over their own identity and message history.

This project, therefore, focuses on the design and implementation of a secure decentralized messaging application called **NodeChat**. The system is developed as a working academic prototype that demonstrates local identity management, direct peer messaging, group conversations, message-state feedback, and encrypted communication in a peer-to-peer environment.

### 1.2 Statement of the Problem

Despite the widespread use of modern messaging platforms, important problems remain in the dominant centralized architecture:

1. **Privacy concerns:** users often rely on external providers to route, manage, or store communication data and metadata.
2. **Centralized control:** a service operator can suspend accounts, restrict access, or act as an intermediary in communication.
3. **Single point of failure:** if the service infrastructure fails or is blocked, communication is interrupted for all users.
4. **Attractive attack surface:** centralized systems can become high-value targets for surveillance, data theft, or other security attacks.
5. **Weak local ownership:** users may not fully control their identity, stored history, or trust relationships within the application.

These problems show the need for a messaging system that is designed around decentralization, secure communication, and stronger user control over local data and identity.

### 1.3 Aim and Objectives of the Study

The main aim of this study is to design and implement a secure decentralized messaging application that supports direct peer-to-peer communication without relying on a traditional central messaging server.

The specific objectives are:

1. To design a peer-to-peer messaging architecture for direct and group communication.
2. To implement secure message protection using modern cryptographic techniques.
3. To develop a user-friendly interface for identity setup, contact management, and conversation handling.
4. To store identity, contacts, groups, and message history locally on the user's device.
5. To provide visible message-state feedback such as queued, sent, delivered, and read.
6. To evaluate the practical behavior of the proposed system as an academic prototype.

### 1.4 Significance of the Study

This study is significant in several ways.

Academically, it contributes to the study of decentralized communication systems, secure messaging design, and practical peer-to-peer application structure. It demonstrates that a final-year project can integrate networking, persistence, security, and user interface design into a single coherent system.

Technically, the project shows how Rust, Slint, SQLite, and the Iroh networking stack can be combined to produce a functioning secure messaging prototype. It also highlights the practical value of explicit message-state handling and user-controlled trust.

From a cybersecurity perspective, the study explores how decentralization and local data ownership can reduce dependence on centralized infrastructure and improve privacy-aware communication design.

### 1.5 Scope and Delimitation of the Study

This project is limited to the design and implementation of a decentralized messaging application with the following scope:

1. Local identity creation and optional PIN-based application protection.
2. Direct peer-to-peer conversations between users.
3. Invitation-based group conversations.
4. Local persistence of identity, contacts, groups, and message history.
5. Secure session establishment and encrypted direct/group message transport.
6. Message-state handling for queued, sent, delivered, and read states.

The study does not attempt to build a full commercial messaging platform. It does not cover voice calls, video calls, cloud account recovery, advanced moderation systems, or large-scale operational deployment concerns.

### 1.6 Limitations of the Study

The following limitations apply to the current system:

1. In restrictive network conditions, direct peer-to-peer connectivity may require relay assistance.
2. The current app is designed as a serious academic prototype and not as a complete consumer-scale platform.
3. Identity portability and recovery workflows are limited because the model prioritizes local ownership.
4. Group management is functional but intentionally narrow in scope.
5. Queued delivery is practical but should not be presented as a guaranteed store-and-forward system.

### 1.7 Organisation of the Work

This project is organized into five chapters.

- **Chapter One** introduces the study, states the problem, identifies the aims and objectives, and explains the scope and limitations.
- **Chapter Two** reviews related literature on communication models, peer-to-peer networking, messaging protocols, cryptography, and local data storage.
- **Chapter Three** presents the system analysis and design of the proposed solution.
- **Chapter Four** explains the implementation of the system and evaluates the achieved result.
- **Chapter Five** gives the summary, conclusion, recommendations, and suggested future work.

### 1.8 Definition of Terms

**Peer-to-Peer (P2P):** A decentralized network model in which participants communicate directly without depending on a single central server for message exchange.

**Decentralization:** A system design approach in which control and communication are distributed across participants rather than concentrated in one authority.

**End-to-End Encryption (E2EE):** A method of protecting communication so that only the sender and intended receiver can read the message content.

**Node:** A participant device or endpoint in a peer-to-peer network.

**Rust:** A systems programming language focused on memory safety, concurrency, and performance.

**Iroh:** A networking stack used to support peer-to-peer connectivity and transport behavior in the implemented system.

**SQLite:** A lightweight embedded relational database used for local data persistence.

**X25519:** A modern elliptic-curve Diffie-Hellman key agreement mechanism used for shared-secret derivation.

**ChaCha20-Poly1305:** An authenticated encryption algorithm used to provide confidentiality and integrity for message payloads.

**Message Status:** The visible state of a message in the system, such as queued, sent, delivered, or read.

---

## CHAPTER TWO: LITERATURE REVIEW

### 2.0 Introduction

This chapter reviews scholarly and technical ideas relevant to the proposed system. The review focuses on communication network models, peer-to-peer networking, messaging protocols, cryptographic protection, and local data persistence. The purpose is to establish the theoretical and technical background for the project and to show the gap that the present work intends to address.

### 2.1 Communication Network Models

The architecture of a messaging system is strongly influenced by its network model. The two most important models in this context are the **client-server model** and the **peer-to-peer model**.

In the client-server model, clients depend on a central server for coordination, identity handling, service logic, and often data storage. This model is widely used because it simplifies central management, makes enforcement easier, and supports straightforward scaling patterns. However, it also introduces central dependency and risk concentration. If the server becomes unavailable or compromised, the communication system is affected broadly.

The peer-to-peer model distributes communication responsibility among participants. In this model, each peer can act as both a requester and a provider of network interaction. The architecture reduces dependence on a single central control point and can improve resilience and user ownership. Historically, peer-to-peer systems became notable in file sharing, distributed content systems, and later decentralized application design.

For messaging applications, the relevance of the peer-to-peer model lies in its ability to reduce central control over identity and message routing. This makes it an appropriate model for privacy-aware communication research.

### 2.2 Peer-to-Peer Networking and NAT Traversal

A major challenge in peer-to-peer communication is the difficulty of establishing direct connectivity between devices located behind routers and private local networks. Network Address Translation (NAT) is commonly used to map private internal addresses to a public address, but this also complicates unsolicited inbound communication between peers.

To address this challenge, several techniques have been developed. STUN helps a device identify its outward-facing address. TURN provides relay assistance when direct connectivity cannot be achieved. ICE coordinates multiple candidate paths in order to establish the most suitable connection between peers.

Modern peer-to-peer networking stacks build these ideas into higher-level connectivity systems. In this project, the adopted networking layer is intended to support direct communication when possible and relay-assisted behavior when needed. This allows the proposed system to remain usable under practical internet conditions without requiring end users to configure routers manually.

### 2.3 Instant Messaging Protocols

The history of instant messaging has moved from isolated proprietary systems toward open and more flexible communication designs. Early platforms such as ICQ, AIM, and MSN Messenger were largely provider-controlled. Later systems such as XMPP introduced more openness through standardization and federation.

Federated systems improve openness compared with single-vendor centralization, but they still rely on server infrastructure. Fully peer-to-peer approaches go further by attempting to reduce or remove the messaging server from the communication path itself.

The proposed system belongs to this more decentralized direction. It is designed to demonstrate direct peer-based messaging with local identity management, while still preserving usability concerns such as saved contacts, message history, and visible feedback.

### 2.4 Cryptography in Messaging Applications

Security in messaging systems depends not only on connectivity, but also on the cryptographic protection of exchanged data.

Symmetric encryption protects data using a shared secret key. It is efficient and suitable for message payload encryption once communicating parties already share secure key material. In modern secure messaging systems, authenticated encryption is preferred because it protects both confidentiality and integrity.

Asymmetric cryptography allows two parties to establish a shared secret over an untrusted channel. Diffie-Hellman-based techniques are widely used for this purpose. In this project, X25519 is relevant because it offers an efficient and modern key agreement mechanism suitable for secure messaging.

After a shared secret is established, a key derivation process is used to prepare key material for encryption. The resulting key can then be used with an authenticated encryption scheme such as ChaCha20-Poly1305. This combination is widely recognized as strong and practical for software-based secure communication.

### 2.5 End-to-End Encryption in Practice

End-to-end encryption means that message content is protected in a form that only intended communicating parties can decrypt. This idea has become central to modern privacy-focused messaging.

In practice, secure messaging systems differ in complexity. Some employ advanced session-evolution techniques such as ratcheting for forward secrecy and recovery properties. Others implement a more foundational model centered on secure key exchange and encrypted transport. The proposed system belongs to the second category: it demonstrates core secure session establishment and encrypted message handling suitable for an academic prototype.

This is important because the project does not need to reproduce every feature of a large-scale commercial secure messenger in order to remain technically meaningful. What matters is that the implemented security model is coherent, understandable, and aligned with the app's current scope.

### 2.6 Local Data Storage and Persistence

A common weakness of many communication platforms is strong dependence on provider-controlled storage. In contrast, local data storage allows users to retain greater control over identity, contacts, and message history.

SQLite is particularly useful in this context because it is lightweight, embedded, and suitable for desktop and mobile application storage. In the proposed system, local persistence supports continuity between sessions and strengthens the project's data ownership story. It also allows the prototype to behave like a real application rather than a temporary networking demonstration.

### 2.7 Summary of Literature Review

The literature reviewed in this chapter shows a clear progression from centralized communication systems toward more open, decentralized, and privacy-aware approaches. It also shows that practical secure messaging requires more than basic transport; it depends on a suitable network model, workable connectivity strategy, message protection, and reliable local state handling.

The gap addressed by this project lies in building a decentralized messaging application that combines these concerns into one practical software system. The proposed work responds to this gap by implementing a peer-to-peer messaging prototype with local identity ownership, encrypted communication, local persistence, and visible communication state.

---

## CHAPTER THREE: SYSTEM ANALYSIS AND DESIGN

### 3.0 Introduction

This chapter presents the analysis and design of the proposed system. It explains the weakness of the existing approach, the requirements of the new system, the development methodology adopted, and the design of the NodeChat application.

### 3.1 Analysis of the Existing System

The existing communication pattern most users depend on today is the centralized messaging system. In this arrangement, the service provider manages identity registration, server-based routing, storage, and platform access. Although this model is common and often convenient, it presents several design weaknesses for a study focused on privacy and decentralization.

Observed weaknesses include:

1. Dependence on central servers for normal communication flow.
2. Concentration of metadata and service control in one provider.
3. Greater exposure to broad outage when the central system fails.
4. Reduced user ownership over identity and stored communication history.
5. Difficulty defending privacy claims when users must trust a service operator heavily.

These limitations justify the need for a system that shifts more responsibility toward direct peer communication and local ownership.

### 3.2 Analysis of the Proposed System

The proposed system, NodeChat, is designed as a decentralized messaging application in which each user maintains a local identity and communicates with peers through a peer-to-peer networking layer. Rather than treating the application as only a transport experiment, the design includes user interface flows, persistence, message progression, and explicit trust-related state.

The proposed system is intended to provide:

1. A local identity owned by the user.
2. Direct messaging between peers.
3. Group communication through invitation-based onboarding.
4. Local storage of chats, contacts, groups, and message history.
5. Encrypted communication once secure session material is established.
6. User-visible communication states and notices.

The proposed model therefore addresses both technical communication needs and user-facing application needs.

### 3.3 Methodology

The methodology adopted for this work is a **prototype-oriented software development approach**. This is appropriate because the project required repeated design, implementation, refinement, and verification of a working software system rather than a purely theoretical study.

The process followed these broad stages:

1. Problem identification and topic definition.
2. Review of literature on messaging systems, decentralization, and secure communication.
3. Design of the proposed peer-to-peer application architecture.
4. Implementation of the software modules.
5. Integration of storage, networking, security, and user interface behavior.
6. Functional verification and evaluation of the resulting system.

This methodology is suitable for a computer science project because it allows ideas to be tested through an actual implementation.

### 3.4 Functional Requirements

The major functional requirements of the proposed system are as follows:

1. The system shall allow a user to create a local identity.
2. The system shall allow optional PIN-based app protection.
3. The system shall allow a user to add contacts using a ticket or peer identifier.
4. The system shall support direct messaging between peers.
5. The system shall support creation of group conversations.
6. The system shall allow invitation-based onboarding into groups.
7. The system shall store contacts, groups, and messages locally.
8. The system shall show communication state such as queued, sent, delivered, and read.
9. The system shall allow retry of queued work when communication becomes available again.
10. The system shall support confirmation flow for destructive actions such as deletion or reset.

### 3.5 Non-Functional Requirements

The key non-functional requirements include:

1. **Security:** communication should use secure key exchange and authenticated encryption.
2. **Usability:** the interface should make core operations understandable to users.
3. **Responsiveness:** the application should keep the UI responsive while background work continues.
4. **Maintainability:** the software should be structured into clear modules.
5. **Persistence:** user data should survive application restarts.
6. **Portability:** the design should remain suitable for desktop and Android-oriented builds.

### 3.6 Architectural Design of the Proposed System

The architecture of NodeChat is organized into five main layers:

1. **User Interface Layer**
   The user interface is built with Slint and provides screens for identity setup, contact management, chats, settings, group creation, and conversation display.

2. **Bridge Layer**
   The bridge connects the synchronous user interface with the asynchronous backend. It forwards user commands to the runtime and returns backend-generated events to the UI.

3. **Backend Coordination Layer**
   This layer acts as the central coordinator of the application. It handles commands, network events, snapshot building, conversation state updates, and integration between storage and transport.

4. **Peer-to-Peer Transport Layer**
   This layer manages direct peer connectivity, group topic subscriptions, direct message transport, and group broadcast behavior.

5. **Storage and Cryptographic Support Layer**
   SQLite stores identity, peers, groups, and messages. Cryptographic utilities provide shared-secret derivation, key handling, and payload encryption/decryption.

The data flow of the system can be summarized as follows:

1. The user performs an action in the interface.
2. The bridge converts the action into a backend command.
3. The backend processes the command and interacts with storage, transport, or both.
4. Resulting state changes are converted into application events.
5. The user interface receives the events and updates visible state.

This design keeps the user interface separated from the deeper application logic and improves clarity of operation.

### 3.7 Database Design

The system uses SQLite as its local database. The main logical tables in the design are:

1. **Local Identity Table**
   Stores the user's display name, local node identifier, secret key material, endpoint ticket, and optional PIN hash.

2. **Peers Table**
   Stores saved contacts, endpoint tickets, exchanged public key information, and verification status.

3. **Groups Table**
   Stores joined or created groups, group names, descriptions, and symmetric group keys.

4. **Messages Table**
   Stores direct and group messages, sender information, target conversation identifiers, timestamps, status values, and invite-related metadata.

This database design supports message restoration, contact persistence, unread-state calculation, and group continuity across app restarts.

### 3.8 Security Design

The security design of the proposed system includes the following ideas:

1. Identity is stored locally on the device rather than created through a central account system.
2. Direct peer communication uses a handshake that exchanges the material required for secure session readiness.
3. Shared secrets are derived for direct message protection.
4. Authenticated encryption is used to protect confidentiality and integrity of message payloads.
5. Group messaging uses symmetric group-key protection.
6. Manual contact verification is kept separate from transport success or handshake completion.

This distinction between secure-session readiness and trust is one of the strongest design decisions in the project because it makes the security model easier to explain and defend.

### 3.9 User Interface Design

The user interface is designed around practical user flows instead of abstract screens alone. Important flows in the design include:

1. First-run identity creation.
2. Optional unlock flow when local protection is enabled.
3. Contact addition through peer ticket or node identifier.
4. Direct conversation selection and message sending.
5. Group creation and invitation.
6. Contact verification and information view.
7. Retry of queued communication work.
8. Confirmation flow for deletion, reset, and other destructive actions.

The UI also communicates important system state through visible indicators and notices. This includes secure-session readiness cues, online and offline indicators, unread counts, and message-state progression.

### 3.10 Chapter Summary

This chapter has analyzed the weaknesses of the existing centralized messaging approach and presented the design of the proposed decentralized solution. It described the project methodology, the requirements, the architectural structure, the database design, and the security model of the system. These design decisions form the basis for the implementation described in the next chapter.

---

## CHAPTER FOUR: SYSTEM IMPLEMENTATION AND EVALUATION

### 4.0 Introduction

This chapter explains how the proposed system was implemented and evaluates the resulting application. It presents the development tools used, the major implementation areas, the features achieved, and the observed outcome of the system.

### 4.1 Development Tools and Environment

The system was implemented with the following tools and technologies:

| Component | Technology Used | Purpose |
| --- | --- | --- |
| Programming language | Rust | Core application logic |
| UI framework | Slint | Desktop/mobile-oriented interface |
| Networking | Iroh and Iroh Gossip | Peer-to-peer direct and group communication |
| Database | SQLite via `rusqlite` | Local persistence |
| Cryptography | X25519, SHA-256, ChaCha20-Poly1305 | Secure key agreement and message protection |
| Async runtime | Tokio | Background task execution |
| Build tooling | Cargo | Compilation and verification |

The implementation was carried out within a modular Rust codebase, with separate directories for application logic, UI assets, project documentation, Android packaging, and the supporting website layer.

### 4.2 Implementation Overview

The NodeChat implementation is structured into clear modules:

1. **Backend modules**
   These modules process commands, handle network events, manage active conversation state, and coordinate transport and storage behavior.

2. **Transport modules**
   These manage direct peer connections, group subscriptions, protocol framing, and transport-level event generation.

3. **Storage modules**
   These manage database initialization, schema application, and SQL queries for local identity, contacts, groups, and messages.

4. **Cryptographic module**
   This provides helper logic for key generation, shared-secret derivation, and payload encryption/decryption.

5. **User interface modules**
   These define the Slint screens, reusable components, UI models, and bridge logic that connect the interface to the backend runtime.

The implementation follows a command-event pattern. User actions are converted into backend commands, while backend processing produces application events that refresh the interface state.

### 4.3 Implemented Features

The implemented system currently provides the following features:

#### 4.3.1 Local Identity Creation

The application allows a new user to create a local identity by providing a display name. The app may also be protected with an optional PIN. After setup, the identity and related local data are stored on the device.

#### 4.3.2 Direct Peer Messaging

Users can add peers through a shareable connection ticket or peer identifier. Once a contact is saved, the app can initiate direct communication and maintain conversation history locally.

#### 4.3.3 Secure Session Handling

The application establishes secure session material through a handshake process. This provides the key material required for protected direct communication.

#### 4.3.4 Group Messaging

Users can create groups, assign a name and description, and invite contacts through direct communication channels. Joined groups become part of the user's local conversation list.

#### 4.3.5 Message-State Feedback

Messages are tracked through meaningful visible states:

1. **Queued** when the message is stored locally and waiting for a viable transport path.
2. **Sent** when the local application has transmitted the message.
3. **Delivered** when the receiving side has acknowledged or stored the message.
4. **Read** when the message reaches an active-view context in the receiving application.

This makes the app behavior easier to understand and gives the project stronger technical credibility.

#### 4.3.6 Local Persistence

The application stores local identity, peers, groups, and message history in SQLite. This allows the user to reopen the application and continue from an existing local state rather than starting from scratch.

#### 4.3.7 Trust and Verification

The system distinguishes between secure-session readiness and user trust. A handshake does not automatically make a contact trusted. Verification remains a separate saved user action.

### 4.4 Testing and Evaluation

The implemented application was evaluated through practical verification of core flows and successful build validation.

#### 4.4.1 Build Verification

The project successfully completed a Rust build verification pass using:

```bash
cargo check
```

This completed successfully in the development environment and confirmed that the current codebase compiles without type or build errors.

#### 4.4.2 Functional Evaluation

The following behaviors were evaluated against the implemented application logic:

1. Identity creation and local persistence.
2. Optional unlock flow when PIN protection is enabled.
3. Contact addition through ticket or node identifier.
4. Direct conversation loading and message sending.
5. Queued message retry behavior.
6. Group creation and invitation handling.
7. Message-state transitions for queued, sent, delivered, and read.
8. Destructive confirmation paths such as deletion or identity reset.

#### 4.4.3 Observed Outcome

The evaluation showed that the application behaves as a real stateful prototype rather than as a static demonstration. The system coordinates local storage, peer communication, and interface state updates in a consistent way.

Observed strengths include:

1. Clear separation of interface logic from backend processing.
2. Persistent local state across sessions.
3. Meaningful visible message-state progression.
4. Practical handling of direct and group conversations.
5. A defendable security story based on local identity and encrypted communication.

Observed boundaries include:

1. Group workflow is intentionally limited to core invitation and participation behavior.
2. Identity recovery and portability are still basic.
3. The app should be presented as a strong academic prototype, not a finished commercial platform.

### 4.5 Discussion of Results

The implemented result supports the main goal of the study. NodeChat demonstrates that a decentralized messaging application can be designed as a complete application structure with local identity, local persistence, peer communication, encryption-aware behavior, and user-visible state handling.

The project is especially strong in the way it joins multiple concerns into one system:

1. Networking is not isolated from the rest of the app.
2. Security is represented both technically and in the user interaction model.
3. Persistence gives the application continuity and realism.
4. Message-state handling shows that the app models communication progress explicitly.

This makes the project suitable for academic defense because it can be discussed from several perspectives: software engineering, cybersecurity, networking, user interface design, and local data ownership.

### 4.6 Chapter Summary

This chapter described the tools, implementation structure, achieved features, and observed results of the proposed system. It showed that the application compiles successfully and that its core features align with the design objectives stated earlier. The final chapter now presents the overall summary, conclusion, and recommendations.

---

## CHAPTER FIVE: SUMMARY, CONCLUSION AND RECOMMENDATIONS

### 5.0 Introduction

This chapter presents the summary of the study, the conclusion drawn from the work, and recommendations for future improvement.

### 5.1 Summary

This project addressed the problem of dependence on centralized messaging infrastructure by proposing and implementing a secure decentralized messaging application called NodeChat. The study began by identifying the privacy, control, and reliability concerns associated with central server-based messaging systems. A review of related literature showed the relevance of peer-to-peer communication, secure key exchange, authenticated encryption, and local persistence to this problem domain.

Based on this foundation, the project designed a system that combines local identity ownership, peer-to-peer direct communication, invitation-based group messaging, SQLite-based persistence, and visible message-state tracking. The system was implemented using Rust, Slint, Iroh, Tokio, SQLite, X25519, SHA-256, and ChaCha20-Poly1305.

The resulting application demonstrates practical user flows such as identity creation, contact addition, direct chat, group creation, group invitation, queued retry behavior, verification control, and local destructive confirmation paths. The project therefore achieved a functioning and coherent academic prototype.

### 5.2 Conclusion

The study concludes that it is practical to design and implement a secure decentralized messaging application as a real software system within the scope of a final-year computer science project. The implemented result shows that peer-to-peer communication, local identity ownership, encrypted transport, and persistent local state can be integrated into a single understandable application.

NodeChat does not attempt to claim the maturity of a commercial large-scale messaging platform. Instead, its value lies in demonstrating a defensible architecture, realistic application behavior, and a clear security-aware design suitable for academic presentation and future improvement.

### 5.3 Recommendations

Based on the work carried out, the following recommendations are made:

1. Future improvements should strengthen identity export, recovery, and portability.
2. A richer trust verification ceremony should be considered beyond the present manual verification toggle.
3. Group administration features such as member roles and moderation can be added in later versions.
4. More formal performance evaluation and multi-device testing should be carried out.
5. Additional platform hardening and release preparation should be considered for broader deployment.

### 5.4 Suggestions for Further Work

The following areas are suggested for further research and development:

1. Forward-secrecy improvements through session ratcheting techniques.
2. Richer offline delivery strategies for decentralized messaging.
3. Voice and multimedia extensions.
4. Broader usability testing with real users.
5. Expanded Android and cross-platform deployment refinement.

---

## REFERENCES

The reference list below is a working academic reference base for the current draft and should be normalized to the department's required citation style before final submission.

1. Bernstein, D. J. (2006). Curve25519: new Diffie-Hellman speed records.
2. Bernstein, D. J. (2008). ChaCha, a variant of Salsa20.
3. Cohen, B. (2003). Incentives build robustness in BitTorrent.
4. Diffie, W., and Hellman, M. (1976). New directions in cryptography.
5. Marlinspike, M., and Perrin, T. (2016). The Signal Protocol.
6. Owens, M. (2006). The Definitive Guide to SQLite.
7. Rosenberg, J., Mahy, R., Matthews, P., and Wing, D. (2008). Session Traversal Utilities for NAT (STUN).
8. Tanenbaum, A. S., and Wetherall, D. J. (2011). Computer Networks.
9. NodeChat project documentation: architecture, security, message lifecycle, user flows, and limitations.
10. Monoed Africa. Final Year Project Format in Nigeria.
11. Delta State Polytechnic, Ogwashi-Uku. Guidelines for Preparing Student Projects.

