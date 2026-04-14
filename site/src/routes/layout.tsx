/* eslint-disable qwik/jsx-img */
import { component$, Slot } from "@builder.io/qwik";
import { Link } from "@builder.io/qwik-city";
import { navItems, siteMeta } from "../content/site-content";

export default component$(() => {
  return (
    <div class="min-h-screen bg-surface-primary text-text-primary">
      <div class="absolute inset-x-0 top-0 -z-10 h-[520px] bg-[radial-gradient(circle_at_top,_rgba(74,158,232,0.18),_transparent_58%)]" />

      <nav class="sticky top-0 z-50 border-b border-divider/80 bg-surface-primary/88 backdrop-blur-xl">
        <div class="mx-auto flex max-w-6xl items-center justify-between gap-6 px-6 py-4">
          <Link href="/" class="flex items-center gap-3">
            <img
              src="/icons/app_logo_transparent_bg.png"
              alt="NodeChat"
              width={40}
              height={40}
              class="h-10 w-10"
            />
            <div class="flex flex-col">
              <span class="text-lg font-semibold tracking-tight text-text-primary">NodeChat</span>
              <span class="text-xs text-text-tertiary">Peer-to-peer messaging project</span>
            </div>
          </Link>

          <div class="hidden items-center gap-6 text-sm md:flex">
            {navItems.map((item) =>
              item.external ? (
                <a
                  key={item.href}
                  href={item.href}
                  target="_blank"
                  rel="noreferrer"
                  class="text-text-secondary transition-colors hover:text-accent"
                >
                  {item.label}
                </a>
              ) : (
                <Link
                  key={item.href}
                  href={item.href}
                  class="text-text-secondary transition-colors hover:text-accent"
                >
                  {item.label}
                </Link>
              ),
            )}
          </div>
        </div>
      </nav>

      <main class="flex-1">
        <Slot />
      </main>

      <footer class="border-t border-divider/80 bg-surface-secondary/35 py-10">
        <div class="mx-auto grid max-w-6xl gap-8 px-6 md:grid-cols-[1.2fr_0.8fr_0.8fr]">
          <div class="space-y-3">
            <p class="text-lg font-semibold tracking-tight text-text-primary">{siteMeta.name}</p>
            <p class="max-w-md text-sm leading-6 text-text-secondary">{siteMeta.tagline}</p>
            <p class="text-xs text-text-tertiary">
              &copy; {new Date().getFullYear()} NodeChat. Built from the app-first documentation
              set.
            </p>
          </div>

          <div class="space-y-3">
            <p class="text-sm font-semibold uppercase tracking-[0.18em] text-text-tertiary">
              Site
            </p>
            <div class="flex flex-col gap-2 text-sm">
              <Link href="/" class="text-text-secondary transition-colors hover:text-accent">
                Home
              </Link>
              <Link href="/docs" class="text-text-secondary transition-colors hover:text-accent">
                Docs
              </Link>
              <Link href="/about" class="text-text-secondary transition-colors hover:text-accent">
                About
              </Link>
            </div>
          </div>

          <div class="space-y-3">
            <p class="text-sm font-semibold uppercase tracking-[0.18em] text-text-tertiary">
              Repository
            </p>
            <div class="flex flex-col gap-2 text-sm">
              <a
                href={siteMeta.repository}
                target="_blank"
                rel="noreferrer"
                class="text-text-secondary transition-colors hover:text-accent"
              >
                GitHub
              </a>
              <a
                href={`${siteMeta.repository}/issues`}
                target="_blank"
                rel="noreferrer"
                class="text-text-secondary transition-colors hover:text-accent"
              >
                Report an issue
              </a>
              <Link
                href="/contributing"
                class="text-text-secondary transition-colors hover:text-accent"
              >
                Contributing
              </Link>
            </div>
          </div>
        </div>
      </footer>
    </div>
  );
});
