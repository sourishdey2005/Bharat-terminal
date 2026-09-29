// app/page.tsx — Made by Sourish Dey
import dynamic from "next/dynamic";
import Link from "next/link";
import { Download } from "lucide-react";
import { CodeBlock, ComparisonTable } from "@/components/charts";
import { NewsletterForm } from "@/components/forms";
import { Hero } from "@/components/home";
import { Container, SectionHeader } from "@/components/layout";
import { LiveChart } from "@/components/live-chart";
import { SITE } from "@/lib/utils";
import { pageMeta } from "@/lib/seo";
import { ScrollReveal } from "@/components/scroll-animations";

const HorizontalFeatures = dynamic(
  () => import("@/components/horizontal-features").then((mod) => mod.HorizontalFeatures),
  { ssr: false, loading: () => <div className="py-20 sm:py-24" /> }
);

export const metadata = pageMeta({
  title: "Bharat Terminal",
  description: "Free open-source Bloomberg alternative in Rust: 140+ visualizations, real NSE/BSE data. Made by Sourish Dey.",
  path: "/",
});

export default function Home() {
  return (
    <>
      <Hero />

      <section className="border-b border-subtle bg-panel" aria-label="Live demo">
        <Container className="py-16 sm:py-20">
          <ScrollReveal variant="slideLeft">
            <SectionHeader eyebrow="Live demo" title="See it live. Real Yahoo Finance data. No API key." sub="Switch symbols and timeframes — the same bt-data crate that powers the desktop app." />
          </ScrollReveal>
          <ScrollReveal variant="fadeScale" style={{ transitionDelay: "100ms" }}>
            <LiveChart />
          </ScrollReveal>
        </Container>
      </section>

      <HorizontalFeatures />

      <section aria-label="Comparison">
        <Container className="py-20 sm:py-24">
          <ScrollReveal variant="slideLeft">
            <SectionHeader eyebrow="Compare" title="Bharat Terminal vs everyone else" />
          </ScrollReveal>
          <ScrollReveal variant="fadeScale" style={{ transitionDelay: "100ms" }}>
            <ComparisonTable />
          </ScrollReveal>
        </Container>
      </section>

      <section className="border-y border-subtle bg-base" aria-label="Get started">
        <Container className="py-20 sm:py-24">
          <ScrollReveal variant="slideRight">
            <SectionHeader eyebrow="Get started" title="One command. 140 charts." sub="Install the toolchain, run a single command, open your first chart in under two minutes." />
          </ScrollReveal>
          <ScrollReveal variant="fadeScale" style={{ transitionDelay: "100ms" }} className="mx-auto max-w-2xl">
            <CodeBlock lang="bash" code="cargo run --release -p bt-cli -- --symbol RELIANCE.NS --live" />
            <div className="mt-6 text-center">
              <Link href="/download" className="cta-gradient inline-flex min-h-[48px] items-center gap-2 rounded-xl px-8 font-bold shadow-glow hover:brightness-110" aria-label="Download Bharat Terminal">
                <Download size={18} aria-hidden /> Download Bharat Terminal 3.0.0
              </Link>
            </div>
          </ScrollReveal>
        </Container>
      </section>

      <section className="border-y border-subtle bg-base" aria-label="Open source">
        <Container className="py-24 sm:py-28">
          <ScrollReveal variant="fadeScale" className="mx-auto max-w-2xl text-center">
            <SectionHeader eyebrow="Open source" title="Built in the open. MIT licensed." sub="Every visualization, every commit, every roadmap decision — public. By Sourish Dey, for everyone." />
            <div className="mt-2 flex flex-col items-center justify-center gap-3 sm:flex-row">
              <a href={SITE.github} target="_blank" rel="noreferrer" aria-label="Star on GitHub" className="cta-gradient inline-flex min-h-[52px] items-center gap-2 rounded-xl px-9 text-base font-bold shadow-glow hover:brightness-110">★ Star on GitHub</a>
              <Link href="/docs" aria-label="Read the documentation" className="inline-flex min-h-[52px] items-center rounded-xl border border-strong bg-panel px-9 text-base font-semibold text-primary transition-colors hover:border-amber hover:text-amber">Read the docs</Link>
            </div>
          </ScrollReveal>
        </Container>
      </section>

      <section aria-label="Newsletter">
        <Container className="py-20 sm:py-24">
          <ScrollReveal variant="fadeScale" className="text-center">
            <SectionHeader eyebrow="Updates" title="Get updates when we ship new visualizations" />
            <NewsletterForm />
          </ScrollReveal>
        </Container>
      </section>
    </>
  );
}