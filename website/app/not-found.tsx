import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Not found — Bharat Terminal by Sourish Dey",
  authors: [{ name: "Sourish Dey" }],
};

export default function NotFound() {
  return (
    <div className="mx-auto grid min-h-[60vh] w-full max-w-2xl place-items-center px-4 py-24 text-center">
      <div>
        <p className="font-mono text-sm text-amber">404 · BT-ERR-NOT-FOUND</p>
        <h1 className="mt-4 font-display text-5xl font-extrabold">Lost in the order book.</h1>
        <p className="mt-4 text-secondary">This page slipped like a stop-loss. Let&apos;s get you back to the terminal.</p>
        <div className="mt-8 flex justify-center gap-3">
          <a href="/" className="cta-gradient rounded-xl px-6 py-3 font-bold" aria-label="Go home">Go home</a>
          <a href="/docs" className="rounded-xl border border-strong px-6 py-3 text-primary hover:border-amber" aria-label="Open docs">Docs</a>
        </div>
        <p className="mt-8 text-xs text-tertiary">Bharat Terminal — Made by Sourish Dey</p>
      </div>
    </div>
  );
}
