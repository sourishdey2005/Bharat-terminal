import { Container, SectionHeader } from "@/components/layout";
import { Badge } from "@/components/brand";
import { pageMeta } from "@/lib/seo";

export const metadata = pageMeta({ title: "About", description: "About Sourish Dey, creator of Bharat Terminal — free Rust Bloomberg alternative from India.", path: "/about" });

export default function AboutPage() {
  return (
    <Container className="py-16">
      <SectionHeader eyebrow="About" title="Built by Sourish Dey. For everyone." align="left" />
      <div className="grid gap-8 lg:grid-cols-[1fr_320px]">
        <div className="space-y-4 text-secondary">
          <p><span className="font-semibold text-primary">Bharat Terminal</span> started with a simple frustration: professional market tooling costs $24,000 a year. Indian retail investors deserved better — so <span className="font-semibold text-amber">Sourish Dey</span> built it in Rust: 140 visualizations, real Yahoo + Coinbase data, native desktop speed.</p>
          <p>Everything is MIT licensed. No paywalls, no API keys, no accounts. If you can run an <span className="font-mono text-sm">.exe</span>, you can run a terminal.</p>
          <Badge variant="india">🇮🇳 Made in India</Badge>
          <h2 className="pt-4 font-display text-2xl font-bold text-primary">Principles</h2>
          <ul className="list-disc space-y-1 pl-5">
            <li>Free forever beats free trials.</li>
            <li>Performance is a feature (60ms for 10 charts).</li>
            <li>India-first data, global coverage.</li>
            <li>Open source, open roadmap.</li>
          </ul>
        </div>
        <aside className="h-fit rounded-2xl border border-subtle bg-panel p-6 text-center" aria-label="Author card">
          <span className="mx-auto flex h-20 w-20 items-center justify-center rounded-full bg-[rgba(255,176,0,0.15)] font-display text-2xl font-extrabold text-amber" aria-hidden>SD</span>
          <h2 className="mt-4 font-display text-xl font-bold text-primary">Sourish Dey</h2>
          <p className="text-sm text-tertiary">Creator & maintainer</p>
          <a href="https://github.com/sourishdey/bharat-terminal" className="amber-link mt-3 inline-block text-sm font-semibold">github.com/sourishdey →</a>
        </aside>
      </div>
    </Container>
  );
}
