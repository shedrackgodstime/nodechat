import { component$ } from "@builder.io/qwik";
import type { DocumentHead } from "@builder.io/qwik-city";
import { siteMeta } from "../../content/site-content";

export default component$(() => {
  return (
    <div class="px-6 py-16">
      <div class="mx-auto max-w-5xl space-y-8">
        <section class="rounded-[2rem] border border-divider bg-surface-secondary/55 p-8">
          <p class="text-sm font-semibold uppercase tracking-[0.18em] text-text-tertiary">
            Contributing
          </p>
          <h1 class="mt-4 text-4xl font-semibold tracking-tight text-text-primary sm:text-5xl">
            Contributions should strengthen the app, not drift from it.
          </h1>
          <p class="mt-5 max-w-3xl text-base leading-7 text-text-secondary">
            NodeChat welcomes bug reports, fixes, improvements to implementation quality, and
            documentation work based on the current app. Contributions should stay grounded in the
            implemented project and maintain the same professional tone now used across the app,
            docs, and site.
          </p>
        </section>

        <section class="grid gap-6 md:grid-cols-2">
          <article class="rounded-[1.75rem] border border-divider bg-surface-secondary/55 p-7">
            <h2 class="text-2xl font-semibold tracking-tight text-text-primary">
              Useful contribution types
            </h2>
            <ul class="mt-5 space-y-3 text-sm leading-6 text-text-secondary">
              <li>clear bug reports with reproduction steps</li>
              <li>code fixes for real app behavior issues</li>
              <li>improvements to tests, consistency, and maintainability</li>
              <li>documentation updates based on implemented behavior</li>
              <li>UI refinements that match the current project direction</li>
            </ul>
          </article>

          <article class="rounded-[1.75rem] border border-divider bg-surface-secondary/55 p-7">
            <h2 class="text-2xl font-semibold tracking-tight text-text-primary">
              Contribution rules
            </h2>
            <ul class="mt-5 space-y-3 text-sm leading-6 text-text-secondary">
              <li>work from the implemented app, not assumptions</li>
              <li>avoid placeholder copy and unfinished narrative</li>
              <li>keep comments, docs, and UI wording professional</li>
              <li>do not overclaim features that are still outside scope</li>
              <li>prefer focused, reviewable changes</li>
            </ul>
          </article>
        </section>

        <section class="rounded-[2rem] border border-divider bg-surface-secondary/55 p-8">
          <h2 class="text-2xl font-semibold tracking-tight text-text-primary">
            Read the contribution guidance
          </h2>
          <p class="mt-4 text-sm leading-6 text-text-secondary">
            The full contribution guide lives in the repository documentation and should be read
            before making broader changes.
          </p>
          <div class="mt-6 flex flex-col gap-4 sm:flex-row">
            <a
              href={`${siteMeta.repository}/blob/main/docs/contributing.md`}
              target="_blank"
              rel="noreferrer"
              class="rounded-full bg-accent px-6 py-3 text-center text-sm font-semibold text-white transition-colors hover:bg-accent/90"
            >
              Open contributing guide
            </a>
            <a
              href={`${siteMeta.repository}/issues`}
              target="_blank"
              rel="noreferrer"
              class="rounded-full border border-divider px-6 py-3 text-center text-sm font-semibold text-text-secondary transition-colors hover:border-accent hover:text-accent"
            >
              Report an issue
            </a>
          </div>
        </section>
      </div>
    </div>
  );
});

export const head: DocumentHead = {
  title: "Contributing | NodeChat",
  meta: [
    {
      name: "description",
      content:
        "Learn how to contribute to NodeChat in a way that matches the implemented app, the current documentation direction, and the project’s professional tone.",
    },
  ],
};
