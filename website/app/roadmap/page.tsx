import { Container, SectionHeader } from "@/components/layout";
import { ROADMAP } from "@/data/roadmap";
import { pageMeta } from "@/lib/seo";

export const metadata = pageMeta({ title: "Roadmap", description: "Bharat Terminal public roadmap: WASM, screener, backtester. By Sourish Dey.", path: "/roadmap" });

const COLOR: Record<string, string> = { Shipped: "text-profit", Building: "text-amber", Planned: "text-tertiary" };

export default function RoadmapPage() {
  return (
    <Container className="max-w-3xl py-16">
      <SectionHeader eyebrow="Roadmap" title="Where Bharat Terminal goes next" align="left" />
      <ol className="space-y-4">
        {ROADMAP.map((r) => (
          <li key={r.title} className="rounded-2xl border border-subtle bg-panel p-6">
            <div className="flex flex-wrap items-center justify-between gap-2">
              <h2 className="font-semibold text-primary">{r.title}</h2>
              <span className={`text-xs font-bold uppercase tracking-widest ${COLOR[r.status]}`}>{r.status} · {r.quarter}</span>
            </div>
            <p className="mt-2 text-sm text-secondary">{r.description}</p>
          </li>
        ))}
      </ol>
    </Container>
  );
}
