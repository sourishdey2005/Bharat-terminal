import type { Metadata } from "next";
import { SITE } from "./utils";

export function pageMeta(opts: {
  title: string;
  description: string;
  path?: string;
}): Metadata {
  const url = `${SITE.url}${opts.path ?? ""}`;
  const fullTitle = `${opts.title} — Bharat Terminal by Sourish Dey`;
  return {
    title: fullTitle,
    description: opts.description,
    authors: [{ name: SITE.author, url: SITE.github }],
    creator: SITE.author,
    alternates: { canonical: url },
    openGraph: {
      title: fullTitle,
      description: opts.description,
      url,
      siteName: "Bharat Terminal",
      type: "website",
      images: [{ url: `${SITE.url}/og-image.png`, width: 1200, height: 630 }],
    },
    twitter: {
      card: "summary_large_image",
      title: fullTitle,
      description: opts.description,
      creator: "@sourishdey",
      images: [`${SITE.url}/og-image.png`],
    },
  };
}

export function softwareJsonLd() {
  return {
    "@context": "https://schema.org",
    "@graph": [
      {
        "@type": "SoftwareApplication",
        name: "Bharat Terminal",
        author: { "@type": "Person", name: "Sourish Dey" },
        applicationCategory: "FinanceApplication",
        operatingSystem: "Windows, macOS, Linux",
        offers: { "@type": "Offer", price: "0", priceCurrency: "USD" },
        softwareVersion: SITE.version,
      },
      { "@type": "Person", name: "Sourish Dey", url: SITE.github },
    ],
  };
}
