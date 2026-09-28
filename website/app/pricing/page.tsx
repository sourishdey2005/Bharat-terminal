import Link from "next/link";
import { Check } from "lucide-react";
import { Container, SectionHeader } from "@/components/layout";
import { pageMeta } from "@/lib/seo";

export const metadata = pageMeta({ title: "Pricing", description: "Bharat Terminal pricing: always free, MIT licensed. No tiers. By Sourish Dey.", path: "/pricing" });

export default function PricingPage() {
  return (
    <Container className="py-16 text-center">
      <SectionHeader eyebrow="Pricing" title="Always free. No tiers. No catch." sub="One plan: everything, forever, MIT licensed by Sourish Dey." />
      <div className="mx-auto max-w-md rounded-2xl border-2 border-amber bg-panel p-8 shadow-glow">
        <p className="font-display text-5xl font-extrabold text-amber">₹0</p>
        <p className="mt-1 text-sm text-secondary">free forever</p>
        <ul className="mt-6 space-y-2.5 text-left text-sm text-primary">
          {["All 140 visualizations", "Real NSE/BSE + US + crypto data", "Desktop app + CLI", "No API keys, no account", "MIT license, commercial use OK", "Community support"].map((f) => (
            <li key={f} className="flex gap-2"><Check size={16} className="mt-0.5 shrink-0 text-profit" /> {f}</li>
          ))}
        </ul>
        <Link href="/download" aria-label="Download free" className="cta-gradient mt-8 flex min-h-[48px] items-center justify-center rounded-xl font-bold">Download free</Link>
      </div>
    </Container>
  );
}
