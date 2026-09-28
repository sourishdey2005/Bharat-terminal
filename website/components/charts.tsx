"use client";
import { motion } from "framer-motion";
import { Check, Copy } from "lucide-react";
import { useState } from "react";

export function TerminalWindow({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="overflow-hidden rounded-xl border border-subtle bg-panel shadow-lg" role="region" aria-label={title}>
      <div className="flex items-center gap-2 border-b border-subtle bg-elevated px-4 py-3" aria-hidden>
        <span className="h-3 w-3 rounded-full bg-loss" />
        <span className="h-3 w-3 rounded-full bg-amber" />
        <span className="h-3 w-3 rounded-full bg-profit" />
        <span className="ml-3 font-mono text-xs text-tertiary">{title}</span>
      </div>
      <div className="terminal-scroll overflow-x-auto p-5 font-mono text-[13px] leading-6">{children}</div>
    </div>
  );
}

export function CodeBlock({ code, lang = "bash" }: { code: string; lang?: string }) {
  const [copied, setCopied] = useState(false);
  return (
    <div className="relative overflow-hidden rounded-xl border border-subtle bg-void">
      <div className="flex items-center justify-between border-b border-subtle px-4 py-2">
        <span className="font-mono text-xs text-tertiary">{lang}</span>
        <button
          aria-label="Copy code to clipboard"
          onClick={() => { navigator.clipboard.writeText(code); setCopied(true); setTimeout(() => setCopied(false), 1500); }}
          className="inline-flex h-8 items-center gap-1.5 rounded-md px-2 text-xs font-semibold text-amber hover:bg-elevated"
        >
          {copied ? <Check size={14} /> : <Copy size={14} />} {copied ? "Copied" : "Copy"}
        </button>
      </div>
      <pre className="terminal-scroll overflow-x-auto p-4 font-mono text-[13px] leading-6 text-primary"><code>{code}</code></pre>
    </div>
  );
}

export function ComparisonTable() {
  const rows: Array<[string, string, string, string, string]> = [
    ["Price", "Free", "$24k/yr", "$15/mo", "Free"],
    ["Visualizations", "140+", "200+", "100+", "20+"],
    ["Open Source", "✅", "❌", "❌", "❌"],
    ["API Keys Needed", "❌", "✅", "✅", "✅"],
    ["Desktop App", "✅", "✅", "❌", "❌"],
    ["CLI", "✅", "❌", "❌", "❌"],
    ["India Data (NSE/BSE)", "✅", "✅", "✅", "✅"],
  ];
  return (
    <div className="terminal-scroll overflow-x-auto rounded-xl border border-subtle">
      <table className="w-full min-w-[640px] border-collapse bg-panel text-sm">
        <caption className="sr-only">Bharat Terminal vs Bloomberg vs TradingView vs Zerodha Kite</caption>
        <thead className="sticky top-0">
          <tr className="bg-elevated text-left">
            <th scope="col" className="px-5 py-4 font-semibold text-secondary">Feature</th>
            <th scope="col" className="border-x border-amber bg-[rgba(255,176,0,0.10)] px-5 py-4 font-bold text-amber">Bharat Terminal</th>
            <th scope="col" className="px-5 py-4 font-semibold text-secondary">Bloomberg</th>
            <th scope="col" className="px-5 py-4 font-semibold text-secondary">TradingView</th>
            <th scope="col" className="px-5 py-4 font-semibold text-secondary">Kite</th>
          </tr>
        </thead>
        <tbody>
          {rows.map(([f, b, bb, tv, k], i) => (
            <motion.tr
              key={f}
              initial={{ opacity: 0, x: -14 }}
              whileInView={{ opacity: 1, x: 0 }}
              viewport={{ once: true, margin: "-32px" }}
              transition={{ duration: 0.45, delay: Math.min(i * 0.05, 0.3), ease: [0.16, 1, 0.3, 1] }}
              className="border-t border-subtle"
            >
              <th scope="row" className="px-5 py-3 text-left font-medium text-primary">{f}</th>
              <td className="border-x border-amber bg-[rgba(255,176,0,0.06)] px-5 py-3 font-bold text-amber">{b}</td>
              <td className="px-5 py-3 text-secondary">{bb}</td>
              <td className="px-5 py-3 text-secondary">{tv}</td>
              <td className="px-5 py-3 text-secondary">{k}</td>
            </motion.tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export function VizGallery({ tabs }: { tabs: Array<{ id: string; label: string; items: Array<{ name: string; desc: string }> }> }) {
  const [active, setActive] = useState(tabs[0]?.id);
  const [lightbox, setLightbox] = useState<{ name: string; desc: string } | null>(null);
  const tab = tabs.find((t) => t.id === active) ?? tabs[0];
  return (
    <div>
      <div className="mb-6 flex flex-wrap gap-2" role="tablist" aria-label="Visualization families">
        {tabs.map((t) => (
          <button key={t.id} role="tab" aria-selected={active === t.id} onClick={() => setActive(t.id)}
            className={`min-h-[40px] rounded-lg px-4 text-sm font-semibold transition-colors ${active === t.id ? "border-b-2 border-amber text-amber" : "text-secondary hover:text-amber"}`}>
            {t.label}
          </button>
        ))}
      </div>
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {tab.items.map((c) => (
          <button key={c.name} onClick={() => setLightbox(c)} aria-label={`Open ${c.name}`}
            className="group overflow-hidden rounded-xl border border-subtle bg-panel text-left transition-all hover:-translate-y-0.5 hover:border-amber hover:shadow-glow">
            <span className="block bg-void p-4 transition-transform duration-300 group-hover:scale-[1.03]">
              <MiniSpark seed={c.name.length * 7} />
            </span>
            <span className="block p-4">
              <span className="block text-sm font-semibold text-primary">{c.name}</span>
              <span className="mt-1 block text-xs text-tertiary transition-all group-hover:text-secondary">{c.desc}</span>
            </span>
          </button>
        ))}
      </div>
      {lightbox && (
        <div role="dialog" aria-modal="true" aria-label={lightbox.name} className="fixed inset-0 z-[70] grid place-items-center bg-black/80 p-4" onClick={() => setLightbox(null)}>
          <div className="w-full max-w-2xl rounded-2xl border border-subtle bg-panel p-6" onClick={(e) => e.stopPropagation()}>
            <h3 className="font-display text-xl font-bold text-primary">{lightbox.name}</h3>
            <p className="mt-2 text-sm text-secondary">{lightbox.desc}</p>
            <div className="mt-4 rounded-xl bg-void p-4"><MiniSpark seed={lightbox.name.length * 13} big /></div>
            <div className="mt-4 flex justify-end gap-3">
              <a href="/features" className="amber-link text-sm font-semibold">See all 140 →</a>
              <button onClick={() => setLightbox(null)} aria-label="Close preview" className="rounded-lg border border-strong px-4 py-2 text-sm text-primary hover:border-amber">Close</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export function MiniSpark({ seed, big }: { seed: number; big?: boolean }) {
  const pts = Array.from({ length: 24 }, (_, i) => {
    const v = 50 + 30 * Math.sin((i + seed) * 0.7) + ((i * seed) % 17) - 8;
    return `${(i * (big ? 30 : 12)).toFixed(0)},${(120 - v).toFixed(0)}`;
  }).join(" ");
  return (
    <svg viewBox={`0 0 ${big ? 720 : 288} 120`} className="h-24 w-full" role="img" aria-label="Chart preview sparkline">
      <polyline points={pts} fill="none" stroke="#FFB000" strokeWidth="2" />
    </svg>
  );
}

export function CheckX({ v }: { v: string }) {
  if (v === "✅") return <span className="font-bold text-profit" aria-label="yes">✓</span>;
  if (v === "❌") return <span className="font-bold text-loss" aria-label="no">✕</span>;
  return <span>{v}</span>;
}
