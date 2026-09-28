import Link from "next/link";
import { Container, SectionHeader } from "@/components/layout";
import { pageMeta } from "@/lib/seo";

export const metadata = pageMeta({ title: "Blog", description: "Bharat Terminal blog: Rust finance, NSE data, quant charts. By Sourish Dey.", path: "/blog" });

export default function BlogPage() {
  return (
    <Container className="py-16">
      <SectionHeader eyebrow="Blog" title="Notes from the terminal" align="left" />
      <Link href="/blog/launch" className="block rounded-2xl border border-subtle bg-panel p-8 transition-all hover:border-amber hover:shadow-glow">
        <p className="font-mono text-xs text-tertiary">2026-09-28 · Launch · 6 min</p>
        <h2 className="mt-2 font-display text-2xl font-bold text-primary">Introducing Bharat Terminal 3.0</h2>
        <p className="mt-2 text-secondary">Bloomberg power, zero cost: 140 Rust visualizations, real NSE data, native installers. Why I built it and how it renders everything in 842ms.</p>
        <span className="amber-link mt-4 inline-block text-sm font-semibold">Read the launch post →</span>
      </Link>
    </Container>
  );
}
