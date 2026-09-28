// app/docs/companies/page.tsx — Made by Sourish Dey
import { COMPANIES } from "@/data/companies";
import { pageMeta } from "@/lib/seo";

export const metadata = pageMeta({ title: "Supported Companies", description: "78 free symbols: NSE, SENSEX, US stocks, crypto, FX. Bharat Terminal by Sourish Dey.", path: "/docs/companies" });

export default function CompaniesDoc() {
  return (
    <div>
      <p className="font-mono text-xs text-tertiary">Docs / Companies</p>
      <h1 className="mt-2 font-display text-4xl font-extrabold">Supported companies ({COMPANIES.length})</h1>
      <p className="mt-3 text-secondary">NSE India, US equities, indices, crypto, commodities and FX — no API keys.</p>
      <div className="terminal-scroll mt-6 overflow-x-auto rounded-xl border border-subtle">
        <table className="w-full min-w-[560px] text-sm">
          <caption className="sr-only">Supported companies</caption>
          <thead><tr className="bg-elevated text-left text-secondary"><th className="px-4 py-3">Symbol</th><th className="px-4 py-3">Name</th><th className="px-4 py-3">Market</th><th className="px-4 py-3">Sector</th></tr></thead>
          <tbody className="bg-panel">
            {COMPANIES.map((c) => (
              <tr key={c.symbol} className="border-t border-subtle"><td className="px-4 py-2 font-mono text-amber">{c.symbol}</td><td className="px-4 py-2 text-primary">{c.name}</td><td className="px-4 py-2 text-secondary">{c.exchange}</td><td className="px-4 py-2 text-secondary">{c.sector}</td></tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
