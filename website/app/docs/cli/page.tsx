import { CodeBlock } from "@/components/charts";
import { pageMeta } from "@/lib/seo";

export const metadata = pageMeta({ title: "CLI Reference", description: "Bharat Terminal CLI: render 140 free Rust charts from your terminal. By Sourish Dey.", path: "/docs/cli" });

export default function CliDoc() {
  return (
    <div>
      <p className="font-mono text-xs text-tertiary">Docs / CLI</p>
      <h1 className="mt-2 font-display text-4xl font-extrabold">CLI Reference</h1>
      <div className="mt-6 space-y-4">
        <CodeBlock lang="bash" code="bt-cli --symbol RELIANCE.NS --live            # live auto-refresh`nbt-cli --symbol TCS.NS --range 1Y              # yearly history`nbt-cli --list-companies                        # 78 supported symbols`nbt-cli --symbol BTC-USD --theme light           # light charts" />
        <table className="w-full overflow-hidden rounded-xl border border-subtle text-sm">
          <caption className="sr-only">CLI options</caption>
          <thead><tr className="bg-elevated text-left text-secondary"><th className="px-4 py-3">Flag</th><th className="px-4 py-3">Description</th></tr></thead>
          <tbody className="bg-panel">
            {[["--symbol <SYM>", "Symbol, e.g. RELIANCE.NS, AAPL, BTC-USD"], ["--live", "30s auto-refresh loop"], ["--range 1D|1W|1M|3M|6M|1Y|5Y", "History window"], ["--list-companies", "Print 78 supported symbols"], ["--theme dark|light", "Chart theme"], ["--out <dir>", "Output dir (default ./output)"]].map(([f, d]) => (
              <tr key={f} className="border-t border-subtle"><td className="px-4 py-2.5 font-mono text-amber">{f}</td><td className="px-4 py-2.5 text-secondary">{d}</td></tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
