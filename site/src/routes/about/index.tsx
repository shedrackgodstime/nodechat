import { component$ } from "@builder.io/qwik";
import { Link, type DocumentHead } from "@builder.io/qwik-city";
import { siteMeta } from "../../content/site-content";

export default component$(() => {
  return (
    <div class="px-6 py-16">
      <div class="mx-auto grid max-w-6xl gap-8 lg:grid-cols-[0.95fr_1.05fr]">
        <section class="rounded-[2rem] border border-divider bg-surface-secondary/55 p-8">
          <p class="text-sm font-semibold uppercase tracking-[0.18em] text-text-tertiary">About</p>
          <h1 class="mt-4 text-4xl font-semibold tracking-tight text-text-primary sm:text-5xl">
            What NodeChat is trying to achieve.
          </h1>
          <p class="mt-5 text-base leading-7 text-text-secondary">
            NodeChat is a final-year project focused on decentralized messaging as a complete app.
            The goal is not only to demonstrate networking concepts, but to show how identity,
            persistence, security-aware interaction design, and messaging flow can be brought
            together in one coherent system.
          </p>
          <p class="mt-4 text-base leading-7 text-text-secondary">
            The project is strongest when it is described from the implemented application itself.
            That is why the docs, README, and site are now being aligned around the current app
            rather than older placeholder narratives.
          </p>
        </section>

        <section class="space-y-6">
          <div class="rounded-[2rem] border border-divider bg-surface-secondary/55 p-8">
            <p class="text-sm font-semibold uppercase tracking-[0.18em] text-text-tertiary">
              Project identity
            </p>
            <ul class="mt-5 space-y-3 text-sm leading-6 text-text-secondary">
              <li>Peer-to-peer messaging application</li>
              <li>Final-year academic project</li>
              <li>Rust and Slint app with direct chat, group chat, local storage, and secure transport</li>
              <li>Serious prototype, not a finished consumer platform</li>
            </ul>
          </div>

          <div class="rounded-[2rem] border border-divider bg-surface-secondary/55 p-8">
            <p class="text-sm font-semibold uppercase tracking-[0.18em] text-text-tertiary">
              Site direction
            </p>
            <p class="mt-4 text-sm leading-6 text-text-secondary">
              The site is intended to become a focused product-and-docs layer:
            </p>
            <ul class="mt-4 space-y-3 text-sm leading-6 text-text-secondary">
              <li>homepage for project identity and status</li>
              <li>docs landing for structured reading</li>
              <li>supporting pages for about and contribution</li>
            </ul>
            <p class="mt-5 text-sm leading-6 text-text-secondary">
              It should stay disciplined and should not drift into placeholder marketing or claims
              the app does not support.
            </p>
          </div>

          <div class="rounded-[2rem] border border-divider bg-surface-secondary/55 p-8">
            <p class="text-sm font-semibold uppercase tracking-[0.18em] text-text-tertiary">
              Next entry points
            </p>
            <div class="mt-5 flex flex-col gap-4 sm:flex-row">
              <Link
                href="/docs"
                class="rounded-full bg-accent px-6 py-3 text-center text-sm font-semibold text-white transition-colors hover:bg-accent/90"
              >
                Read the docs
              </Link>
              <a
                href={siteMeta.repository}
                target="_blank"
                rel="noreferrer"
                class="rounded-full border border-divider px-6 py-3 text-center text-sm font-semibold text-text-secondary transition-colors hover:border-accent hover:text-accent"
              >
                Visit repository
              </a>
            </div>
          </div>
        </section>
      </div>
    </div>
  );
});

export const head: DocumentHead = {
  title: "About NodeChat | Project Context",
  meta: [
    {
      name: "description",
      content:
        "Learn what NodeChat is, what it is trying to achieve, and how the project is being presented as a serious final-year peer-to-peer messaging application.",
    },
  ],
};
