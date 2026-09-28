import { Container } from "@/components/layout";
import { pageMeta } from "@/lib/seo";

export const metadata = pageMeta({ title: "Introducing Bharat Terminal 3.0", description: "Launch story: free Rust Bloomberg alternative with 140 charts. By Sourish Dey.", path: "/blog/launch" });

export default function LaunchPost() {
  return (
    <Container className="max-w-3xl py-16">
      <p className="font-mono text-xs text-tertiary">Blog / Launch · 2026-09-28 · by Sourish Dey</p>
      <h1 className="mt-3 font-display text-4xl font-extrabold sm:text-5xl">Introducing Bharat Terminal 3.0</h1>
      <p className="mt-4 text-xl text-secondary">Bloomberg power. Zero cost. Made in India.</p>
      <article className="mt-8 space-y-5 text-base leading-7 text-secondary">
        <p>Professional terminals cost more than most Indian salaries. That never made sense to me. So I built <strong className="text-primary">Bharat Terminal</strong> — a free, MIT-licensed terminal in Rust with <strong className="text-primary">140 visualizations</strong> and real market data.</p>
        <h2 className="font-display text-2xl font-bold text-primary">What&apos;s inside</h2>
        <ul className="list-disc space-y-1 pl-5">
          <li>32 candlestick-family charts, 15 Bollinger tools, 25 indicators, full risk/options/statistical suites.</li>
          <li>Yahoo + Coinbase providers with SQLite cache — no API keys, works offline on cached data.</li>
          <li>60ms for 10 charts via SIMD + Rayon; 140 charts in 842ms end-to-end.</li>
          <li>Native installers: MSI, Setup.exe, portable ZIP, DMG, AppImage, deb, rpm.</li>
        </ul>
        <h2 className="font-display text-2xl font-bold text-primary">Why Rust</h2>
        <p>Charts are embarrassingly parallel and allocation-heavy — exactly where Rust shines. Tokio fetches, Rayon renders, and the binary stays small enough to email (don&apos;t).</p>
        <h2 className="font-display text-2xl font-bold text-primary">What&apos;s next</h2>
        <p>WASM web terminal, NSE screener with alerts, and an options backtester. The <a className="amber-link font-semibold" href="/roadmap">public roadmap</a> has dates. Star the repo and shape it.</p>
        <p className="border-t border-subtle pt-6 text-sm text-tertiary">Made by Sourish Dey · MIT Licensed · <a className="amber-link" href="/download">Download 3.0.0 →</a></p>
      </article>
    </Container>
  );
}
