import { Container, SectionHeader } from "@/components/layout";
import { CHANGELOG } from "@/data/roadmap";
import { pageMeta } from "@/lib/seo";

export const metadata = pageMeta({ title: "Changelog", description: "Bharat Terminal version history: v3.0.0 with 140 charts. By Sourish Dey.", path: "/changelog" });

export default function ChangelogPage() {
  return (
    <Container className="max-w-3xl py-16">
      <SectionHeader eyebrow="Changelog" title="Version history" align="left" />
      <ol className="space-y-6">
        {CHANGELOG.map((c) => (
          <li key={c.version} className="rounded-2xl border border-subtle bg-panel p-6">
            <div className="flex items-baseline justify-between gap-3">
              <h2 className="font-display text-xl font-bold text-amber">v{c.version}</h2>
              <time className="font-mono text-xs text-tertiary">{c.date}</time>
            </div>
            <ul className="mt-3 list-disc space-y-1 pl-5 text-sm text-secondary">{c.notes.map((n) => <li key={n}>{n}</li>)}</ul>
          </li>
        ))}
      </ol>
    </Container>
  );
}
