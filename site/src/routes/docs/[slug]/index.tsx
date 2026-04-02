import { component$ } from "@builder.io/qwik";
import { routeLoader$ } from "@builder.io/qwik-city";

export const useDocContent = routeLoader$(async ({ params }) => {
  const slug = params.slug;
  const repoOwner = "shedrackgodstime";
  const repoName = "nodechat";
  const rawUrl = `https://raw.githubusercontent.com/${repoOwner}/${repoName}/main/docs/${slug}.md`;

  try {
    const response = await fetch(rawUrl);
    if (!response.ok) return null;
    const content = await response.text();
    return content;
  } catch {
    return null;
  }
});

export default component$(() => {
  const content = useDocContent();

  if (!content.value) {
    return (
      <div class="max-w-6xl mx-auto px-6 py-12">
        <a href="/docs" class="text-accent hover:underline">
          ← Back to docs
        </a>
        <h1 class="text-2xl font-bold text-text-primary mt-4">Document not found</h1>
      </div>
    );
  }

  const renderMarkdown = (md: string) => {
    const lines = md.split("\n");
    const elements: string[] = [];
    let inCodeBlock = false;
    let codeContent = "";

    for (const line of lines) {
      if (line.startsWith("```")) {
        if (!inCodeBlock) {
          inCodeBlock = true;
          codeContent = "";
        } else {
          elements.push(`<pre class="bg-surface-tertiary p-4 rounded-lg overflow-x-auto mb-4"><code class="text-sm text-text-primary">${escapeHtml(codeContent)}</code></pre>`);
          inCodeBlock = false;
        }
        continue;
      }

      if (inCodeBlock) {
        codeContent += line + "\n";
        continue;
      }

      if (line.startsWith("# ")) {
        elements.push(`<h1 class="text-3xl font-bold text-text-primary mb-6">${escapeHtml(line.slice(2))}</h1>`);
      } else if (line.startsWith("## ")) {
        elements.push(`<h2 class="text-2xl font-semibold text-text-primary mt-8 mb-4">${escapeHtml(line.slice(3))}</h2>`);
      } else if (line.startsWith("### ")) {
        elements.push(`<h3 class="text-xl font-semibold text-text-primary mt-6 mb-3">${escapeHtml(line.slice(4))}</h3>`);
      } else if (line.startsWith("- ")) {
        elements.push(`<li class="text-text-secondary ml-4 mb-2">${escapeHtml(line.slice(2))}</li>`);
      } else if (line.match(/^\d+\.\s/)) {
        elements.push(`<li class="text-text-secondary ml-4 mb-2">${escapeHtml(line.replace(/^\d+\.\s/, ""))}</li>`);
      } else if (line.trim() === "") {
        elements.push("<br/>");
      } else {
        elements.push(`<p class="text-text-secondary mb-4">${escapeHtml(line)}</p>`);
      }
    }

    return elements.join("");
  };

  return (
    <div class="max-w-6xl mx-auto px-6 py-12">
      <a href="/docs" class="text-accent hover:underline mb-6 inline-block">
        ← Back to all docs
      </a>
      <article class="prose prose-invert max-w-none">
        <div
          class="bg-surface-secondary rounded-lg p-8 border border-divider"
          dangerouslySetInnerHTML={renderMarkdown(content.value)}
        />
      </article>
    </div>
  );
});

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}