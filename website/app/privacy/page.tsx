import { Container } from "@/components/layout";
import { pageMeta } from "@/lib/seo";

export const metadata = pageMeta({ title: "Privacy", description: "Bharat Terminal privacy: no tracking by default. By Sourish Dey.", path: "/privacy" });

export default function PrivacyPage() {
  return (
    <Container className="max-w-3xl py-16">
      <h1 className="font-display text-4xl font-extrabold">Privacy policy</h1>
      <p className="mt-2 font-mono text-xs text-tertiary">Effective 2026-09-28 · Made by Sourish Dey</p>
      <div className="mt-6 space-y-4 text-secondary">
        <p>The desktop app collects nothing by default. Market data is fetched directly from Yahoo/Coinbase; no account, no telemetry.</p>
        <p>This website uses privacy-friendly Vercel Analytics (aggregated, no cookies for tracking). The newsletter stores only your email via Resend; unsubscribe anytime.</p>
        <p>Contact: privacy via the <a className="amber-link font-semibold" href="/contact">contact form</a>.</p>
      </div>
    </Container>
  );
}
