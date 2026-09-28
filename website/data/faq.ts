export interface Faq { q: string; a: string; category: string }

export const FAQS: Faq[] = [
  { q: "Is Bharat Terminal really free?", a: "Yes. MIT licensed by Sourish Dey. No tiers, no subscriptions, no limits — forever.", category: "General" },
  { q: "Do I need API keys?", a: "No. Market data comes from Yahoo Finance and Coinbase public endpoints with a local SQLite cache.", category: "Data" },
  { q: "Which platforms are supported?", a: "Windows 10+ (.exe / .msi / portable), macOS 11+ (.dmg / Homebrew), Linux (AppImage / .deb / .rpm / AUR / Snap).", category: "Install" },
  { q: "How many visualizations are included?", a: "140 across 9 families: Candlestick (32), Bollinger (15), Indicators (25), Risk (15), Options (15), Statistical (10), Order Flow (10), India (10), Bloomberg-style (8).", category: "General" },
  { q: "Does it cover NSE/BSE data?", a: "Yes. 40+ NSE symbols plus NIFTY 50, NIFTY Bank, SENSEX, USD/INR and India-specific charts.", category: "Data" },
  { q: "How fast is it?", a: "Rust + SIMD + Rayon renders 10 charts in ~60ms and all 140 in under a second on a typical laptop.", category: "Performance" },
  { q: "Is it open source?", a: "Yes — MIT. Star and fork github.com/sourishdey/bharat-terminal. Contributions welcome.", category: "General" },
  { q: "How do I verify downloads?", a: "Each release ships SHA256 checksums and a GPG signature. See /download verification section.", category: "Install" },
  { q: "Can I use it for commercial research?", a: "Yes, MIT permits commercial use. Market data remains subject to Yahoo/Coinbase terms.", category: "Legal" },
  { q: "Who built this?", a: "Sourish Dey — Bharat Terminal is designed, built and maintained by Sourish Dey.", category: "General" },
];
