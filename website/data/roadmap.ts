export interface RoadmapItem { title: string; description: string; status: "Shipped" | "Building" | "Planned"; quarter: string }
export const ROADMAP: RoadmapItem[] = [
  { title: "140-visualization engine", description: "Full Rust catalog with 842ms batch render.", status: "Shipped", quarter: "Q3 2026" },
  { title: "Native installers", description: "MSI, Inno Setup, portable ZIP, DMG, AppImage.", status: "Shipped", quarter: "Q3 2026" },
  { title: "Website + docs hub", description: "This Next.js site: 20 pages, SEO, live demo.", status: "Shipped", quarter: "Q3 2026" },
  { title: "WASM web terminal", description: "Run the Rust engine in-browser, no install.", status: "Building", quarter: "Q4 2026" },
  { title: "Screener + alerts", description: "NSE screener with price/volume alert rules.", status: "Building", quarter: "Q4 2026" },
  { title: "Options strategy backtester", description: "Payoff + Greeks backtests on NIFTY weeklies.", status: "Planned", quarter: "Q1 2027" },
  { title: "Mobile companion", description: "Read-only watchlist + chart share app.", status: "Planned", quarter: "Q2 2027" },
];

export interface ChangelogEntry { version: string; date: string; notes: string[] }
export const CHANGELOG: ChangelogEntry[] = [
  { version: "3.0.0", date: "2026-09-28", notes: ["140 visualizations, SQLite cache, MSI + portable + AppImage installers", "Full CLI with --live and --list-companies", "Dark/light desktop themes", "Official website launch"] },
  { version: "2.1.0", date: "2026-08-15", notes: ["78-company dropdown with search", "30s live auto-refresh", "Error toasts, no-console Windows binary"] },
  { version: "2.0.0", date: "2026-07-01", notes: ["Real Yahoo + Coinbase data", "25 visualizations", "Full CLI"] },
  { version: "1.0.0", date: "2026-05-10", notes: ["Initial release, 10 synthetic-data charts"] },
];

export interface Testimonial { quote: string; author: string; role: string; initials: string }
export const TESTIMONIALS: Testimonial[] = [
  { quote: "Finally, a free Bloomberg alternative that actually works.", author: "Retail Trader", role: "Mumbai", initials: "RT" },
  { quote: "The 140 charts saved me \u20B950,000/year on subscriptions.", author: "Portfolio Manager", role: "Bengaluru", initials: "PM" },
  { quote: "Rust performance is insane. Loads in 60ms.", author: "Quant Developer", role: "Delhi", initials: "QD" },
];
