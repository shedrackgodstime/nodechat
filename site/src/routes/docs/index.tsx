import { component$ } from "@builder.io/qwik";
import { Link, type DocumentHead } from "@builder.io/qwik-city";
import { groupedDocs, siteMeta } from "../../content/site-content";

export default component$(() => {
  return (
    <div class="px-6 py-16">
      <div class="mx-auto max-w-6xl">
        <div class="max-w-3xl space-y-5">
          <p class="text-sm font-semibold uppercase tracking-[0.18em] text-text-tertiary">
            Documentation
          </p>
          <h1 class="text-4xl font-semibold tracking-tight text-text-primary sm:text-5xl">
            The app-first source of truth for NodeChat.
          </h1>
          <p class="text-base leading-7 text-text-secondary sm:text-lg">
            The documentation is organized around the implemented application, not older placeholder
            claims. These documents support project defense now and will later feed the public site
            more directly.
          </p>
        </div>

        <div class="mt-14 space-y-10">
          {Object.entries(groupedDocs).map(([category, entries]) => (
            <section key={category} class="space-y-5">
              <div class="flex items-end justify-between gap-4 border-b border-divider pb-4">
                <div>
                  <h2 class="text-2xl font-semibold tracking-tight text-text-primary">{category}</h2>
                  <p class="mt-2 text-sm leading-6 text-text-secondary">
                    {category === "Core Product Docs"
                      ? "Start here for the strongest explanation of what NodeChat is and what it currently does."
                      : category === "Technical Reference Docs"
                        ? "Use these when you need the technical model behind the app’s behavior."
                        : "These documents support contribution, maintenance, and future site growth."}
                  </p>
                </div>
              </div>

              <div class="grid gap-5 md:grid-cols-2 xl:grid-cols-3">
                {entries.map((doc) => (
                  <Link
                    key={doc.slug}
                    href={`/docs/${doc.slug}`}
                    class="group rounded-[1.5rem] border border-divider bg-surface-secondary/60 p-6 transition-all hover:-translate-y-0.5 hover:border-accent/45"
                  >
                    <p class="text-lg font-semibold tracking-tight text-text-primary transition-colors group-hover:text-accent">
                      {doc.title}
                    </p>
                    <p class="mt-3 text-sm leading-6 text-text-secondary">{doc.description}</p>
                    <p class="mt-5 text-xs font-semibold uppercase tracking-[0.18em] text-text-tertiary">
                      Read inside site
                    </p>
                  </Link>
                ))}
              </div>
            </section>
          ))}
        </div>

        <div class="mt-16 rounded-[2rem] border border-divider bg-surface-secondary/55 p-8">
          <h2 class="text-2xl font-semibold tracking-tight text-text-primary">
            Recommended reading order
          </h2>
          <ol class="mt-5 space-y-3 text-sm leading-6 text-text-secondary">
            <li>1. Overview</li>
            <li>2. Features</li>
            <li>3. User Flows</li>
            <li>4. Limitations</li>
            <li>5. Security</li>
            <li>6. Message Lifecycle</li>
            <li>7. Architecture</li>
          </ol>
          <p class="mt-6 text-sm leading-6 text-text-secondary">
            This order matches the current project direction: product and behavior first, then
            deeper technical reference.
          </p>
          <p class="mt-6 text-sm leading-6 text-text-secondary">
            If you need the raw repository files, you can still browse them on{" "}
            <a
              href={`${siteMeta.repository}/tree/main/docs`}
              target="_blank"
              rel="noreferrer"
              class="text-accent transition-colors hover:text-accent/80"
            >
              GitHub
            </a>
            .
          </p>
        </div>
      </div>
    </div>
  );
});

export const head: DocumentHead = {
  title: "NodeChat Docs | Current Documentation Set",
  meta: [
    {
      name: "description",
      content:
        "Read the current NodeChat documentation set: overview, features, user flows, limitations, security, message lifecycle, architecture, and contribution guidance.",
    },
  ],
};
