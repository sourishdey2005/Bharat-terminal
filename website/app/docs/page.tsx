import Link from "next/link";
import { CodeBlock } from "@/components/charts";
import { pageMeta } from "@/lib/seo";

export const metadata = pageMeta({ title: "Docs", description: "Bharat Terminal docs: free Rust Bloomberg alternative guides, CLI, NSE data. By Sourish Dey.", path: "/docs" });

export default function DocsHome() {
  return (
    <div>
      <p className="font-mono text-xs text-tertiary">Docs / Overview</p>
      <h1 className="mt-2 font-display text-4xl font-extrabold">Documentation</h1>
      <p className="mt-3 text-secondary">Render your first chart in under 2 minutes. Free forever, no API keys.</p>
      <div className="mt-6"><CodeBlock lang="bash" code="cargo run --release -p bt-cli -- --symbol RELIANCE.NS --live" /></div>
      <div className="mt-6 grid gap-3 sm:grid-cols-2">
        {[["Installation", "/docs/install", "Windows, macOS, Linux in 3 steps."], ["CLI Reference", "/docs/cli", "Every flag with examples."], ["Companies", "/docs/companies", "78 symbols: NSE, US, crypto."], ["Data API", "/docs/api", "Yahoo + Coinbase providers."]].map(([t, h, d]) => (
          <Link key={h} href={h} className="rounded-xl border border-subtle bg-panel p-5 transition-all hover:border-amber hover:shadow-glow">
            <span className="font-semibold text-primary">{t}</span>
            <span className="mt-1 block text-sm text-secondary">{d}</span>
          </Link>
        ))}
      </div>
    </div>
  );
}
