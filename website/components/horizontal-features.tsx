"use client";

import { useLayoutEffect, useRef, useState } from "react";
import { FeatureCard } from "./home";
import { Container, SectionHeader } from "./layout";
import { StaggerItem } from "./scroll-animations";

const FEATURES = [
  { icon: "📈", title: "140+ Visualizations", desc: "Candlesticks, Bollinger, Ichimoku, IV surfaces, Monte Carlo, more.", href: "/features" },
  { icon: "📊", title: "Real Market Data", desc: "Yahoo, Coinbase, NSE/BSE. No API keys. SQLite cache.", href: "/docs/api" },
  { icon: "⚙️", title: "Built in Rust", desc: "60ms for 10 charts. SIMD, Tokio, Rayon. Native performance.", href: "/docs" },
  { icon: "💻", title: "Native Windows App", desc: "Windows .exe — installer, portable ZIP, and MSI builds.", href: "/download" },
  { icon: "🌍", title: "100% Free Forever", desc: "MIT license. No subscriptions. No tiers. No limits.", href: "/pricing" },
  { icon: "🇮🇳", title: "Made in India", desc: "Built by Sourish Dey. For Indian and global markets.", href: "/about" },
];

export function HorizontalFeatures() {
  const [isClient, setIsClient] = useState(false);
  const wrapperRef = useRef<HTMLDivElement>(null);

  useLayoutEffect(() => {
    setIsClient(true);
  }, []);

  if (!isClient) {
    return (
      <section className="relative" aria-label="Why Bharat Terminal">
        <Container className="py-20 sm:py-24">
          <SectionHeader
            eyebrow="Why Bharat Terminal"
            title="Everything a terminal should be"
            sub="Fast, free, and honest. No upsells hiding behind the charts."
          />
          <div className="flex gap-6 pt-12 overflow-x-auto pb-8 snap-x snap-mandatory" role="list" aria-label="Feature cards">
            {FEATURES.map((f, i) => (
              <StaggerItem key={f.title}>
                <div style={{ flexShrink: 0, width: "340px", scrollSnapAlign: "start" }}>
                  <FeatureCard {...f} index={i} />
                </div>
              </StaggerItem>
            ))}
          </div>
        </Container>
      </section>
    );
  }

  return (
    <section className="relative" aria-label="Why Bharat Terminal">
      <Container className="py-20 sm:py-24">
        <SectionHeader
          eyebrow="Why Bharat Terminal"
          title="Everything a terminal should be"
          sub="Fast, free, and honest. No upsells hiding behind the charts."
        />
        <div
          ref={wrapperRef}
          className="flex gap-6 pt-12 overflow-x-auto snap-x snap-mandatory pb-8"
          role="list"
          aria-label="Feature cards"
        >
          {FEATURES.map((f, i) => (
            <StaggerItem key={f.title}>
              <div style={{ flexShrink: 0, width: "340px", scrollSnapAlign: "start" }}>
                <FeatureCard {...f} index={i} />
              </div>
            </StaggerItem>
          ))}
        </div>
      </Container>
    </section>
  );
}