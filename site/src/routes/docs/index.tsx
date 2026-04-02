import { component$ } from "@builder.io/qwik";
import { routeLoader$ } from "@builder.io/qwik-city";

interface DocItem {
  slug: string;
  title: string;
  description: string;
}

export const useDocsList = routeLoader$(async () => {
  const repoOwner = "shedrackgodstime";
  const repoName = "nodechat";
  const docsUrl = `https://api.github.com/repos/${repoOwner}/${repoName}/contents/docs`;

  try {
    const response = await fetch(docsUrl, {
      headers: {
        Accept: "application/vnd.github.v3+json",
      },
    });

    if (!response.ok) return [];

    const files: any[] = await response.json();
    const docs: DocItem[] = [];

    for (const file of files) {
      if (file.name.endsWith(".md")) {
        const slug = file.name.replace(".md", "");
        const rawUrl = `https://raw.githubusercontent.com/${repoOwner}/${repoName}/main/docs/${file.name}`;
        const contentRes = await fetch(rawUrl);
        const content = await contentRes.text();
        
        const titleMatch = content.match(/^#\s+(.+)$/m);
        const title = titleMatch ? titleMatch[1] : slug;

        const firstParaMatch = content.match(/^##\s*\n([^#]+)/m) || content.match(/^#.*\n([^#]+)/m);
        const description = firstParaMatch ? firstParaMatch[1].trim().slice(0, 100) : slug;

        docs.push({ slug, title, description });
      }
    }

    return docs.sort((a, b) => a.title.localeCompare(b.title));
  } catch {
    return [];
  }
});

export default component$(() => {
  const docsList = useDocsList();

  return (
    <div class="max-w-6xl mx-auto px-6 py-12">
      <h1 class="text-3xl font-bold text-text-primary mb-4">Documentation</h1>
      <p class="text-text-secondary mb-12">
        Learn how to build, run, and use NodeChat.
      </p>

      {docsList.value.length === 0 ? (
        <p class="text-text-tertiary">No documentation available yet.</p>
      ) : (
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          {docsList.value.map((doc) => (
            <a
              key={doc.slug}
              href={`/docs/${doc.slug}`}
              class="group p-6 bg-surface-secondary rounded-lg border border-divider hover:border-accent transition-all"
            >
              <h2 class="text-xl font-semibold text-text-primary mb-2 group-hover:text-accent transition-colors">
                {doc.title}
              </h2>
              <p class="text-text-secondary text-sm">
                {doc.description}
              </p>
            </a>
          ))}
        </div>
      )}
    </div>
  );
});