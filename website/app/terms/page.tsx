import { Container } from "@/components/layout";
import { pageMeta } from "@/lib/seo";

export const metadata = pageMeta({ title: "Terms", description: "Bharat Terminal terms: MIT software, market-data disclaimers. By Sourish Dey.", path: "/terms" });

export default function TermsPage() {
  return (
    <Container className="max-w-3xl py-16">
      <h1 className="font-display text-4xl font-extrabold">Terms of service</h1>
      <p className="mt-2 font-mono text-xs text-tertiary">Effective 2026-09-28 · Made by Sourish Dey</p>
      <div className="mt-6 space-y-4 text-secondary">
        <p>Bharat Terminal is MIT-licensed software by Sourish Dey, provided &quot;as is&quot; without warranty.</p>
        <p>Charts are for education and research — not financial advice. Market data from Yahoo/Coinbase may be delayed and remains subject to their terms.</p>
        <p>Do not abuse public data endpoints; caching is built in for exactly this reason.</p>
      </div>
    </Container>
  );
}
