import { CodeBlock } from "@/components/charts";
import { pageMeta } from "@/lib/seo";

export const metadata = pageMeta({ title: "Data Provider API", description: "bt-data crate: Yahoo + Coinbase providers, SQLite cache, Rust traits. By Sourish Dey.", path: "/docs/api" });

export default function ApiDoc() {
  return (
    <div>
      <p className="font-mono text-xs text-tertiary">Docs / API</p>
      <h1 className="mt-2 font-display text-4xl font-extrabold">Data provider API</h1>
      <p className="mt-3 text-secondary">Yahoo Finance + Coinbase with SQLite cache and synthetic fallback. Implement the trait for custom feeds.</p>
      <div className="mt-6 space-y-4">
        <CodeBlock lang="rust" code={`#[async_trait]\npub trait DataProvider {\n    async fn candles(&self, symbol: &str, range: Range) -> Result<OhlcvSeries>;\n}\n\npub struct Candle { pub ts: i64, pub open: f64, pub high: f64, pub low: f64, pub close: f64, pub volume: f64 }\npub struct OhlcvSeries { pub symbol: String, pub candles: Vec<Candle> }`} />
        <CodeBlock lang="rust" code={`let yahoo = YahooProvider::cached("./data/cache.db");\nlet series = yahoo.candles("RELIANCE.NS", Range::M1).await?;`} />
      </div>
    </div>
  );
}
