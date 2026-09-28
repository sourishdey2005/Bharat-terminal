// app/faq/page.tsx — Made by Sourish Dey
import { Container, SectionHeader } from "@/components/layout";
import { FAQS } from "@/data/faq";
import { pageMeta } from "@/lib/seo";

export const metadata = pageMeta({ title: "FAQ", description: "Bharat Terminal FAQ: free forever, no API keys, NSE data. By Sourish Dey.", path: "/faq" });

export default function FaqPage() {
  return (
    <Container className="max-w-3xl py-16">
      <SectionHeader eyebrow="FAQ" title="Questions, answered" align="left" />
      <div className="space-y-3">
        {FAQS.map((f) => (
          <details key={f.q} className="group rounded-xl border border-subtle bg-panel p-5 open:border-amber">
            <summary className="cursor-pointer font-semibold text-primary marker:text-amber">{f.q}</summary>
            <p className="mt-2 text-sm text-secondary">{f.a}</p>
          </details>
        ))}
      </div>
    </Container>
  );
}
