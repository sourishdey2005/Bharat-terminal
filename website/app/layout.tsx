import { Analytics } from "@vercel/analytics/react";
import { SpeedInsights } from "@vercel/speed-insights/next";
import type { Metadata, Viewport } from "next";
import { Manrope, JetBrains_Mono } from "next/font/google";
import { Footer } from "@/components/footer";
import { BackToTop, ScrollProgress } from "@/components/home";
import { Navbar } from "@/components/navbar";
import { Providers } from "@/components/providers";
import { SITE } from "@/lib/utils";
import { softwareJsonLd } from "@/lib/seo";
import "./globals.css";

const manrope = Manrope({
  subsets: ["latin"],
  variable: "--font-manrope",
  display: "swap",
  weight: "variable",
});
const jetbrains = JetBrains_Mono({
  subsets: ["latin"],
  variable: "--font-jetbrains-mono",
  display: "swap",
});

export const metadata: Metadata = {
  metadataBase: new URL(SITE.url),
  title: { default: "Bharat Terminal by Sourish Dey — Bloomberg power. Zero cost.", template: "%s — Bharat Terminal by Sourish Dey" },
  description: "Free open-source Bloomberg alternative in Rust: 140+ visualizations, real NSE/BSE + US + crypto data. Made by Sourish Dey.",
  authors: [{ name: "Sourish Dey", url: SITE.github }],
  creator: "Sourish Dey",
  keywords: ["bloomberg alternative", "free trading terminal", "rust finance", "quantitative finance", "candlestick charts", "technical analysis", "indian stock market", "NSE BSE data", "open source trading"],
  openGraph: { siteName: "Bharat Terminal", type: "website", images: [{ url: "/og-image.png", width: 1200, height: 630 }] },
  twitter: { card: "summary_large_image", creator: "@sourishdey", images: ["/og-image.png"] },
};

export const viewport: Viewport = { themeColor: "#050505", width: "device-width", initialScale: 1 };

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" suppressHydrationWarning className={`${manrope.variable} ${jetbrains.variable}`}>
      <body className="min-h-screen bg-[var(--void)] text-[var(--ink)]">
        <script type="application/ld+json" dangerouslySetInnerHTML={{ __html: JSON.stringify(softwareJsonLd()) }} />
        <Providers>
          <a href="#main" className="skip-link">Skip to content</a>
          <ScrollProgress />
          <div id="top" />
          <Navbar />
          <main id="main">{children}</main>
          <Footer />
          <BackToTop />
        </Providers>
        <Analytics />
        <SpeedInsights />
      </body>
    </html>
  );
}