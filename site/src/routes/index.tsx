/* eslint-disable qwik/jsx-img */
import { component$ } from "@builder.io/qwik";
import { Link, type DocumentHead } from "@builder.io/qwik-city";
import { homepageFeatures, siteMeta } from "../content/site-content";

export default component$(() => {
  return (
    <div class="pb-24">
      <section class="px-6 pb-16 pt-18 sm:pt-24">
        <div class="mx-auto grid max-w-6xl gap-14 lg:grid-cols-[1.1fr_0.9fr] lg:items-center">
          <div class="space-y-8">
            <div class="inline-flex items-center gap-2 rounded-full border border-divider bg-surface-secondary/70 px-4 py-2 text-xs font-semibold uppercase tracking-[0.18em] text-text-tertiary">
              Final-year project
            </div>

            <div class="space-y-5">
              <h1 class="max-w-4xl text-5xl font-semibold tracking-tight text-text-primary sm:text-6xl">
                Peer-to-peer messaging with a clearer product story.
              </h1>
              <p class="max-w-2xl text-lg leading-8 text-text-secondary sm:text-xl">
                {siteMeta.summary}
              </p>
            </div>

            <div class="flex flex-col gap-4 sm:flex-row">
              <Link
                href="/docs"
                class="rounded-full bg-accent px-7 py-3 text-center text-sm font-semibold text-white transition-transform duration-200 hover:-translate-y-0.5 hover:bg-accent/90"
              >
                Read the docs
              </Link>
              <a
                href={siteMeta.repository}
                target="_blank"
                rel="noreferrer"
                class="rounded-full border border-divider bg-surface-secondary/70 px-7 py-3 text-center text-sm font-semibold text-text-secondary transition-colors hover:border-accent hover:text-accent"
              >
                View repository
              </a>
            </div>

            <div class="grid gap-4 pt-3 sm:grid-cols-3">
              <div class="rounded-3xl border border-divider bg-surface-secondary/50 p-5">
                <p class="text-xs font-semibold uppercase tracking-[0.18em] text-text-tertiary">
                  Identity
                </p>
                <p class="mt-3 text-sm leading-6 text-text-secondary">
                  Local ownership, local protection, and connection tickets for peer onboarding.
                </p>
              </div>
              <div class="rounded-3xl border border-divider bg-surface-secondary/50 p-5">
                <p class="text-xs font-semibold uppercase tracking-[0.18em] text-text-tertiary">
                  Messaging
                </p>
                <p class="mt-3 text-sm leading-6 text-text-secondary">
                  Direct chat, group chat, and visible message states across the app.
                </p>
              </div>
              <div class="rounded-3xl border border-divider bg-surface-secondary/50 p-5">
                <p class="text-xs font-semibold uppercase tracking-[0.18em] text-text-tertiary">
                  Trust
                </p>
                <p class="mt-3 text-sm leading-6 text-text-secondary">
                  Secure session readiness and manual verification are kept separate.
                </p>
              </div>
            </div>
          </div>

          <div class="relative">
            <div class="absolute inset-0 rounded-[2rem] bg-[linear-gradient(140deg,rgba(74,158,232,0.2),rgba(52,199,116,0.08),transparent)] blur-2xl" />
            <div class="relative overflow-hidden rounded-[2rem] border border-divider bg-surface-secondary/80 p-8 shadow-[0_20px_80px_rgba(0,0,0,0.28)]">
              <div class="mb-8 flex items-center gap-4">
                <div class="rounded-2xl bg-accent-soft p-3">
                  <img
                    src="/icons/app_logo_with_name_transparent_bg.png"
                    alt="NodeChat"
                    width={180}
                    height={180}
                    class="w-36"
                  />
                </div>
              </div>

              <div class="space-y-5">
                <div class="rounded-2xl border border-divider bg-surface-primary/65 p-5">
                  <p class="text-sm font-semibold text-text-primary">What the app says it is</p>
                  <p class="mt-2 text-sm leading-6 text-text-secondary">
                    A peer-to-peer messaging application with local identity, secure transport,
                    direct messaging, group messaging, and visible app state.
                  </p>
                </div>

                <div class="grid gap-4 sm:grid-cols-2">
                  <div class="rounded-2xl border border-divider bg-surface-primary/55 p-5">
                    <p class="text-sm font-semibold text-text-primary">Current scope</p>
                    <p class="mt-2 text-sm leading-6 text-text-secondary">
                      Serious academic prototype with real application behavior.
                    </p>
                  </div>
                  <div class="rounded-2xl border border-divider bg-surface-primary/55 p-5">
                    <p class="text-sm font-semibold text-text-primary">Documentation-first site</p>
                    <p class="mt-2 text-sm leading-6 text-text-secondary">
                      The site follows the app and docs, not placeholder claims.
                    </p>
                  </div>
                </div>

                <div class="rounded-2xl border border-divider bg-surface-primary/55 p-5">
                  <p class="text-sm font-semibold text-text-primary">Why this matters</p>
                  <p class="mt-2 text-sm leading-6 text-text-secondary">
                    NodeChat is positioned as a full app-level study of decentralized messaging,
                    not just a networking demo or a UI mockup.
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      <section class="px-6 py-18">
        <div class="mx-auto max-w-6xl">
          <div class="mb-10 max-w-2xl">
            <p class="text-sm font-semibold uppercase tracking-[0.18em] text-text-tertiary">
              Core features
            </p>
            <h2 class="mt-3 text-3xl font-semibold tracking-tight text-text-primary sm:text-4xl">
              Built from the app as it exists now.
            </h2>
            <p class="mt-4 text-base leading-7 text-text-secondary">
              The site should reflect the implemented NodeChat application: direct conversations,
              group conversations, local persistence, message-state handling, and a clearer trust
              model.
            </p>
          </div>

          <div class="grid gap-5 md:grid-cols-2 xl:grid-cols-3">
            {homepageFeatures.map((feature) => (
              <article
                key={feature.title}
                class="rounded-[1.75rem] border border-divider bg-surface-secondary/60 p-6 transition-colors hover:border-accent/40"
              >
                <p class="text-lg font-semibold tracking-tight text-text-primary">{feature.title}</p>
                <p class="mt-3 text-sm leading-6 text-text-secondary">{feature.description}</p>
              </article>
            ))}
          </div>
        </div>
      </section>

      <section class="px-6 py-18">
        <div class="mx-auto grid max-w-6xl gap-8 lg:grid-cols-[0.95fr_1.05fr]">
          <div class="rounded-[2rem] border border-divider bg-surface-secondary/55 p-8">
            <p class="text-sm font-semibold uppercase tracking-[0.18em] text-text-tertiary">
              Why NodeChat matters
            </p>
            <h2 class="mt-4 text-3xl font-semibold tracking-tight text-text-primary">
              More than a transport experiment.
            </h2>
            <p class="mt-4 text-base leading-7 text-text-secondary">
              NodeChat combines user interface design, local persistence, peer-to-peer transport,
              security-oriented interaction design, and message lifecycle handling inside one
              consistent application.
            </p>
            <p class="mt-4 text-base leading-7 text-text-secondary">
              That makes it easier to defend academically and easier to turn into a coherent public
              story later through the docs and the site.
            </p>
          </div>

          <div class="rounded-[2rem] border border-divider bg-surface-secondary/55 p-8">
            <p class="text-sm font-semibold uppercase tracking-[0.18em] text-text-tertiary">
              Project status
            </p>
            <h2 class="mt-4 text-3xl font-semibold tracking-tight text-text-primary">
              Honest scope, stronger presentation.
            </h2>
            <ul class="mt-5 space-y-4 text-sm leading-6 text-text-secondary">
              <li>NodeChat is a working peer-to-peer messaging app, not a placeholder concept.</li>
              <li>It should be presented as a strong academic prototype, not as a finished consumer platform.</li>
              <li>The docs now serve as the primary source of truth for the site and the defense story.</li>
            </ul>

            <div class="mt-7 flex flex-col gap-4 sm:flex-row">
              <Link
                href="/about"
                class="rounded-full border border-divider px-6 py-3 text-center text-sm font-semibold text-text-secondary transition-colors hover:border-accent hover:text-accent"
              >
                About the project
              </Link>
              <Link
                href="/contributing"
                class="rounded-full border border-divider px-6 py-3 text-center text-sm font-semibold text-text-secondary transition-colors hover:border-accent hover:text-accent"
              >
                Contribution guide
              </Link>
            </div>
          </div>
        </div>
      </section>
    </div>
  );
});

export const head: DocumentHead = {
  title: "NodeChat | Peer-to-Peer Messaging Project",
  meta: [
    {
      name: "description",
      content:
        "NodeChat is a peer-to-peer messaging project with local identity, direct chat, group chat, secure transport, and app-first documentation.",
    },
  ],
};
