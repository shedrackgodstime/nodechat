import { component$ } from "@builder.io/qwik";
import {
  Link,
  routeLoader$,
  type DocumentHead,
  type StaticGenerateHandler,
} from "@builder.io/qwik-city";
import { docs, findDocBySlug, getAdjacentDocs } from "../../../content/site-content";

const rawDocs = import.meta.glob("../../../../../docs/*.md", {
  query: "?raw",
  import: "default",
  eager: true,
}) as Record<string, string>;

interface LoadedDoc {
  slug: string;
  title: string;
  description: string;
  category: string;
  content: string;
}

export const useDocContent = routeLoader$(async ({ params }) => {
  const doc = findDocBySlug(params.slug);
  if (!doc) {
    return null;
  }

  const relativePath = doc.repoPath.replace(/^\/docs\//, "");
  const entry = Object.entries(rawDocs).find(([key]) => key.endsWith(`/${relativePath}`));
  const content = entry?.[1];

  if (!content) {
    return null;
  }

  return {
    slug: doc.slug,
    title: doc.title,
    description: doc.description,
    category: doc.category,
    content,
  } satisfies LoadedDoc;
});

export const onStaticGenerate: StaticGenerateHandler = async () => {
  return {
    params: docs.map((doc) => ({ slug: doc.slug })),
  };
};

export default component$(() => {
  const loaded = useDocContent();

  if (!loaded.value) {
    return (
      <div class="px-6 py-16">
        <div class="mx-auto max-w-4xl rounded-[2rem] border border-divider bg-surface-secondary/55 p-8">
          <Link href="/docs" class="text-sm font-semibold text-accent transition-colors hover:text-accent/80">
            ← Back to docs
          </Link>
          <h1 class="mt-5 text-3xl font-semibold tracking-tight text-text-primary">
            Document not found
          </h1>
          <p class="mt-4 text-sm leading-6 text-text-secondary">
            The requested document is not part of the current site documentation set.
          </p>
        </div>
      </div>
    );
  }

  const adjacent = getAdjacentDocs(loaded.value.slug);
  const rendered = renderMarkdown(loaded.value.content);

  return (
    <div class="px-6 py-16">
      <div class="mx-auto max-w-5xl">
        <Link
          href="/docs"
          class="text-sm font-semibold text-accent transition-colors hover:text-accent/80"
        >
          ← Back to docs
        </Link>

        <div class="mt-6 rounded-[2rem] border border-divider bg-surface-secondary/55 p-8 sm:p-10">
          <p class="text-sm font-semibold uppercase tracking-[0.18em] text-text-tertiary">
            {loaded.value.category}
          </p>
          <h1 class="mt-4 text-4xl font-semibold tracking-tight text-text-primary sm:text-5xl">
            {loaded.value.title}
          </h1>
          <p class="mt-5 max-w-3xl text-base leading-7 text-text-secondary">
            {loaded.value.description}
          </p>
        </div>

        <article
          class="doc-content mt-8 rounded-[2rem] border border-divider bg-surface-secondary/45 p-8 sm:p-10"
          dangerouslySetInnerHTML={rendered}
        />

        <div class="mt-8 grid gap-4 sm:grid-cols-2">
          {adjacent.previous ? (
            <Link
              href={`/docs/${adjacent.previous.slug}`}
              class="rounded-[1.5rem] border border-divider bg-surface-secondary/50 p-6 transition-colors hover:border-accent/40"
            >
              <p class="text-xs font-semibold uppercase tracking-[0.18em] text-text-tertiary">
                Previous
              </p>
              <p class="mt-3 text-lg font-semibold tracking-tight text-text-primary">
                {adjacent.previous.title}
              </p>
            </Link>
          ) : (
            <div />
          )}

          {adjacent.next ? (
            <Link
              href={`/docs/${adjacent.next.slug}`}
              class="rounded-[1.5rem] border border-divider bg-surface-secondary/50 p-6 text-left transition-colors hover:border-accent/40"
            >
              <p class="text-xs font-semibold uppercase tracking-[0.18em] text-text-tertiary">
                Next
              </p>
              <p class="mt-3 text-lg font-semibold tracking-tight text-text-primary">
                {adjacent.next.title}
              </p>
            </Link>
          ) : (
            <div />
          )}
        </div>
      </div>
    </div>
  );
});

export const head: DocumentHead = {
  title: "NodeChat Doc | In-Site Documentation",
  meta: [
    {
      name: "description",
      content:
        "Read NodeChat documentation directly inside the site, including overview, features, user flows, limitations, security, message lifecycle, and architecture.",
    },
  ],
};

function renderMarkdown(markdown: string): string {
  const lines = markdown.split("\n");
  const html: string[] = [];
  let inCodeBlock = false;
  let codeLines: string[] = [];
  let listType: "ul" | "ol" | null = null;
  let paragraphLines: string[] = [];

  const flushParagraph = () => {
    if (paragraphLines.length === 0) return;
    const text = paragraphLines.join(" ").trim();
    if (text) {
      html.push(`<p>${renderInline(text)}</p>`);
    }
    paragraphLines = [];
  };

  const closeList = () => {
    if (listType) {
      html.push(listType === "ul" ? "</ul>" : "</ol>");
      listType = null;
    }
  };

  for (const rawLine of lines) {
    const line = rawLine.trimEnd();

    if (line.startsWith("```")) {
      flushParagraph();
      closeList();
      if (!inCodeBlock) {
        inCodeBlock = true;
        codeLines = [];
      } else {
        html.push(
          `<pre><code>${escapeHtml(codeLines.join("\n"))}</code></pre>`,
        );
        inCodeBlock = false;
      }
      continue;
    }

    if (inCodeBlock) {
      codeLines.push(rawLine);
      continue;
    }

    if (line === "") {
      flushParagraph();
      closeList();
      continue;
    }

    if (line.startsWith("# ")) {
      flushParagraph();
      closeList();
      html.push(`<h1>${renderInline(line.slice(2))}</h1>`);
      continue;
    }

    if (line.startsWith("## ")) {
      flushParagraph();
      closeList();
      html.push(`<h2>${renderInline(line.slice(3))}</h2>`);
      continue;
    }

    if (line.startsWith("### ")) {
      flushParagraph();
      closeList();
      html.push(`<h3>${renderInline(line.slice(4))}</h3>`);
      continue;
    }

    if (line.startsWith("- ")) {
      flushParagraph();
      if (listType !== "ul") {
        closeList();
        html.push("<ul>");
        listType = "ul";
      }
      html.push(`<li>${renderInline(line.slice(2))}</li>`);
      continue;
    }

    if (/^\d+\.\s/.test(line)) {
      flushParagraph();
      if (listType !== "ol") {
        closeList();
        html.push("<ol>");
        listType = "ol";
      }
      html.push(`<li>${renderInline(line.replace(/^\d+\.\s/, ""))}</li>`);
      continue;
    }

    closeList();
    paragraphLines.push(line);
  }

  flushParagraph();
  closeList();

  if (inCodeBlock) {
    html.push(`<pre><code>${escapeHtml(codeLines.join("\n"))}</code></pre>`);
  }

  return html.join("");
}

function renderInline(text: string): string {
  let output = escapeHtml(text);
  output = output.replace(/`([^`]+)`/g, "<code>$1</code>");
  output = output.replace(
    /\[([^\]]+)\]\(([^)]+)\)/g,
    '<a href="$2" target="_blank" rel="noreferrer">$1</a>',
  );
  return output;
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}
