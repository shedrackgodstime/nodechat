export interface NavItem {
  href: string;
  label: string;
  external?: boolean;
}

export interface FeatureItem {
  title: string;
  description: string;
}

export interface DocEntry {
  slug: string;
  title: string;
  description: string;
  category: "Core Product Docs" | "Technical Reference Docs" | "Project Support Docs";
  repoPath: string;
}

export const siteMeta = {
  name: "NodeChat",
  tagline: "Peer-to-peer messaging with local identity, secure transport, and clear app state.",
  summary:
    "NodeChat is a final-year project built in Rust and Slint. It demonstrates how decentralized messaging can be delivered as a complete application with direct chat, group chat, local persistence, and a clearer trust model.",
  repository: "https://github.com/shedrackgodstime/nodechat",
  website: "https://nodechat.pages.dev",
};

export const navItems: NavItem[] = [
  { href: "/", label: "Home" },
  { href: "/docs", label: "Docs" },
  { href: "/about", label: "About" },
  { href: "/contributing", label: "Contribute" },
  { href: siteMeta.repository, label: "GitHub", external: true },
];

export const homepageFeatures: FeatureItem[] = [
  {
    title: "Local Identity",
    description:
      "Each installation owns its own identity, connection ticket, and local app state.",
  },
  {
    title: "Direct Messaging",
    description:
      "Peers can exchange one-to-one messages through a secure session established by the app.",
  },
  {
    title: "Group Conversations",
    description:
      "Groups are created locally, invitations move through direct messaging, and conversations run over peer-to-peer group transport.",
  },
  {
    title: "Message State Tracking",
    description:
      "The interface reflects queued, sent, delivered, and read progress instead of hiding transport state.",
  },
  {
    title: "Manual Verification",
    description:
      "Secure session readiness and user trust are treated as separate concepts.",
  },
  {
    title: "Local Persistence",
    description:
      "Identity, contacts, groups, and message history are stored locally so the app behaves like a real software system.",
  },
];

export const docs: DocEntry[] = [
  {
    slug: "overview",
    title: "Overview",
    description: "Defines what NodeChat is, what it currently does, and what scope it should claim.",
    category: "Core Product Docs",
    repoPath: "/docs/overview.md",
  },
  {
    slug: "features",
    title: "Features",
    description: "Describes the main user-facing capabilities of the current app in product terms.",
    category: "Core Product Docs",
    repoPath: "/docs/features.md",
  },
  {
    slug: "user-flows",
    title: "User Flows",
    description: "Explains how users move through setup, contacts, messaging, groups, and destructive actions.",
    category: "Core Product Docs",
    repoPath: "/docs/user-flows.md",
  },
  {
    slug: "limitations",
    title: "Limitations",
    description: "States the current project boundaries so the app and site do not overclaim.",
    category: "Core Product Docs",
    repoPath: "/docs/limitations.md",
  },
  {
    slug: "security",
    title: "Security",
    description: "Explains local identity, session establishment, encryption, and trust semantics.",
    category: "Technical Reference Docs",
    repoPath: "/docs/security.md",
  },
  {
    slug: "message-lifecycle",
    title: "Message Lifecycle",
    description: "Describes how messages move through queued, sent, delivered, and read states.",
    category: "Technical Reference Docs",
    repoPath: "/docs/message-lifecycle.md",
  },
  {
    slug: "architecture",
    title: "Architecture",
    description: "Gives a concise view of the implemented app layers and runtime flow.",
    category: "Technical Reference Docs",
    repoPath: "/docs/architecture.md",
  },
  {
    slug: "contributing",
    title: "Contributing",
    description: "Explains how to contribute in a way that matches the project’s standards and tone.",
    category: "Project Support Docs",
    repoPath: "/docs/contributing.md",
  },
  {
    slug: "docs-plan",
    title: "Docs Plan",
    description: "Records the documentation structure and the intended direction for future site expansion.",
    category: "Project Support Docs",
    repoPath: "/docs/DOCS_PLAN.md",
  },
];

export const orderedDocs = docs;

export function findDocBySlug(slug: string) {
  return docs.find((doc) => doc.slug === slug) ?? null;
}

export function getAdjacentDocs(slug: string) {
  const index = orderedDocs.findIndex((doc) => doc.slug === slug);
  if (index === -1) {
    return { previous: null, next: null };
  }

  return {
    previous: index > 0 ? orderedDocs[index - 1] : null,
    next: index < orderedDocs.length - 1 ? orderedDocs[index + 1] : null,
  };
}

export const groupedDocs = docs.reduce<Record<DocEntry["category"], DocEntry[]>>(
  (acc, doc) => {
    acc[doc.category].push(doc);
    return acc;
  },
  {
    "Core Product Docs": [],
    "Technical Reference Docs": [],
    "Project Support Docs": [],
  },
);
