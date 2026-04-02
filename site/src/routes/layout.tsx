import { component$, Slot } from "@builder.io/qwik";
import { Link } from "@builder.io/qwik-city";

export default component$(() => {
  return (
    <div class="min-h-screen flex flex-col">
      <nav class="fixed top-0 w-full bg-surface-primary/80 backdrop-blur-md border-b border-divider z-50">
        <div class="max-w-6xl mx-auto px-6 py-4 flex items-center justify-between">
          <Link href="/" class="flex items-center gap-3">
            <img
              src="/icons/app_logo_transparent_bg.png"
              alt="NodeChat"
              width={40}
              height={40}
              class="w-10 h-10"
            />
            <span class="text-xl font-bold text-text-primary">NodeChat</span>
          </Link>
          <div class="flex items-center gap-8">
            <a href="/#features" class="text-text-secondary hover:text-accent transition-colors">
              Features
            </a>
            <a href="/#team" class="text-text-secondary hover:text-accent transition-colors">
              Team
            </a>
            <Link href="/docs" class="text-text-secondary hover:text-accent transition-colors">
              Docs
            </Link>
          </div>
        </div>
      </nav>
      <main class="flex-1 pt-16">
        <Slot />
      </main>
      <footer class="border-t border-divider py-8">
        <div class="max-w-6xl mx-auto px-6 flex flex-col md:flex-row items-center justify-between gap-4">
          <p class="text-text-tertiary text-sm">
            &copy; {new Date().getFullYear()} NodeChat. Built with Slint.
          </p>
          <div class="flex items-center gap-6">
            <a
              href="https://github.com"
              target="_blank"
              class="text-text-secondary hover:text-accent text-sm transition-colors"
            >
              GitHub
            </a>
          </div>
        </div>
      </footer>
    </div>
  );
});