import { ContactForm } from "@/components/forms";
import { Container, SectionHeader } from "@/components/layout";
import { pageMeta } from "@/lib/seo";

export const metadata = pageMeta({ title: "Contact", description: "Contact Sourish Dey about Bharat Terminal — free Rust Bloomberg alternative.", path: "/contact" });

export default function ContactPage() {
  return (
    <Container className="max-w-xl py-16">
      <SectionHeader eyebrow="Contact" title="Talk to a human" sub="Maintained by Sourish Dey. Replies within 2 business days." align="left" />
      <ContactForm />
    </Container>
  );
}
