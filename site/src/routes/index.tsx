import { component$ } from "@builder.io/qwik";
import type { DocumentHead } from "@builder.io/qwik-city";

export default component$(() => {
  return (
    <>
      {/* Hero Section */}
      <section class="min-h-[90vh] flex items-center justify-center px-6 py-20">
        <div class="max-w-4xl mx-auto text-center">
          <div class="mb-8">
            <img
              src="/icons/app_logo_with_name_transparent_bg.png"
              alt="NodeChat"
              width={200}
              height={200}
              class="mx-auto w-40 sm:w-48 md:w-56"
            />
          </div>
          <h1 class="text-4xl sm:text-5xl md:text-6xl font-bold text-text-primary mb-6">
            Secure Decentralized Chat
          </h1>
          <p class="text-lg sm:text-xl text-text-secondary mb-10 max-w-2xl mx-auto">
            Peer-to-peer messaging with end-to-end encryption. No servers, no tracking—just direct communication.
          </p>
          <div class="flex flex-col sm:flex-row items-center justify-center gap-4">
            <button class="px-8 py-3 bg-accent text-white font-semibold rounded-pill hover:bg-accent/90 transition-colors cursor-pointer">
              Download
            </button>
            <a href="#features" class="px-8 py-3 border border-surface-tertiary text-text-secondary font-semibold rounded-pill hover:border-accent hover:text-accent transition-colors">
              Learn More
            </a>
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section id="features" class="py-20 px-6 bg-surface-secondary/50">
        <div class="max-w-6xl mx-auto">
          <h2 class="text-3xl font-bold text-text-primary text-center mb-12">
            Why NodeChat?
          </h2>
          <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-8">
            {/* Feature 1 */}
            <div class="p-6 bg-surface-secondary rounded-lg border border-divider hover:border-accent/50 transition-colors">
              <div class="w-12 h-12 bg-accent/20 rounded-lg flex items-center justify-center mb-4">
                <svg class="w-6 h-6 text-accent" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                </svg>
              </div>
              <h3 class="text-xl font-semibold text-text-primary mb-2">True P2P</h3>
              <p class="text-text-secondary text-sm">
                No central servers. Messages go directly between peers using Iroh and Pkarr.
              </p>
            </div>

            {/* Feature 2 */}
            <div class="p-6 bg-surface-secondary rounded-lg border border-divider hover:border-accent/50 transition-colors">
              <div class="w-12 h-12 bg-accent-success/20 rounded-lg flex items-center justify-center mb-4">
                <svg class="w-6 h-6 text-accent-success" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                </svg>
              </div>
              <h3 class="text-xl font-semibold text-text-primary mb-2">E2E Encrypted</h3>
              <p class="text-text-secondary text-sm">
                X25519 key exchange + ChaCha20-Poly1305. Your messages, your keys.
              </p>
            </div>

            {/* Feature 3 */}
            <div class="p-6 bg-surface-secondary rounded-lg border border-divider hover:border-accent/50 transition-colors">
              <div class="w-12 h-12 bg-accent-warning/20 rounded-lg flex items-center justify-center mb-4">
                <svg class="w-6 h-6 text-accent-warning" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
                </svg>
              </div>
              <h3 class="text-xl font-semibold text-text-primary mb-2">Actor Model</h3>
              <p class="text-text-secondary text-sm">
                Slint UI stays responsive. All network and crypto ops run async.
              </p>
            </div>

            {/* Feature 4 */}
            <div class="p-6 bg-surface-secondary rounded-lg border border-divider hover:border-accent/50 transition-colors">
              <div class="w-12 h-12 bg-accent-danger/20 rounded-lg flex items-center justify-center mb-4">
                <svg class="w-6 h-6 text-accent-danger" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                </svg>
              </div>
              <h3 class="text-xl font-semibold text-text-primary mb-2">Group Chat</h3>
              <p class="text-text-secondary text-sm">
                Decentralized groups with symmetric key encryption via iroh-gossip.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* Tech Stack Section */}
      <section class="py-16 px-6">
        <div class="max-w-4xl mx-auto text-center">
          <h2 class="text-2xl font-bold text-text-primary mb-8">Built With</h2>
          <div class="flex flex-wrap items-center justify-center gap-8">
            <div class="flex items-center gap-2 text-text-secondary">
              <span class="text-2xl font-bold">Rust</span>
            </div>
            <div class="flex items-center gap-2 text-text-secondary">
              <span class="text-2xl font-bold">Slint</span>
            </div>
            <div class="flex items-center gap-2 text-text-secondary">
              <span class="text-2xl font-bold">Iroh</span>
            </div>
            <div class="flex items-center gap-2 text-text-secondary">
              <span class="text-2xl font-bold">Pkarr</span>
            </div>
          </div>
        </div>
      </section>

      {/* Team Section */}
      <section id="team" class="py-20 px-6 bg-surface-secondary/30">
        <div class="max-w-6xl mx-auto">
          <h2 class="text-3xl font-bold text-text-primary text-center mb-4">
            The Team
          </h2>
          <p class="text-text-secondary text-center mb-12">
            Two students building the future of decentralized messaging.
          </p>
          <div class="grid grid-cols-1 md:grid-cols-2 gap-8 max-w-3xl mx-auto">
            {/* Team Member 1 */}
            <div class="flex items-start gap-4 p-6 bg-surface-secondary rounded-lg border border-divider">
              <div class="w-16 h-16 bg-accent rounded-full flex items-center justify-center text-white text-xl font-bold shrink-0">
                K
              </div>
              <div>
                <h3 class="text-xl font-semibold text-text-primary">Kristency</h3>
                <p class="text-accent text-sm mb-2">Lead Developer</p>
                <p class="text-text-secondary text-sm">
                  Focused on P2P networking, cryptography, and system architecture.
                </p>
              </div>
            </div>

            {/* Team Member 2 */}
            <div class="flex items-start gap-4 p-6 bg-surface-secondary rounded-lg border border-divider">
              <div class="w-16 h-16 bg-accent-success rounded-full flex items-center justify-center text-white text-xl font-bold shrink-0">
                M
              </div>
              <div>
                <h3 class="text-xl font-semibold text-text-primary">Micheal</h3>
                <p class="text-accent-success text-sm mb-2">UI/UX Lead</p>
                <p class="text-text-secondary text-sm">
                  Responsible for Slint UI design, user experience, and frontend integration.
                </p>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section class="py-20 px-6">
        <div class="max-w-3xl mx-auto text-center">
          <h2 class="text-3xl font-bold text-text-primary mb-4">
            Ready to try it?
          </h2>
          <p class="text-text-secondary mb-8">
            Download NodeChat today and experience truly decentralized messaging.
          </p>
          <button class="px-8 py-3 bg-accent text-white font-semibold rounded-pill hover:bg-accent/90 transition-colors cursor-pointer">
            Get Started
          </button>
        </div>
      </section>
    </>
  );
});

export const head: DocumentHead = {
  title: "NodeChat — Secure Decentralized Chat",
  meta: [
    {
      name: "description",
      content: "Peer-to-peer chat application with end-to-end encryption. No servers, just direct messaging.",
    },
  ],
};