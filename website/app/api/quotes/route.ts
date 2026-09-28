import { NextResponse } from "next/server";
import { fetchYahooCandles } from "@/lib/data";

export async function GET(req: Request) {
  const { searchParams } = new URL(req.url);
  const symbol = searchParams.get("symbol") ?? "RELIANCE.NS";
  const tf = searchParams.get("tf") ?? "1M";
  try {
    const candles = await fetchYahooCandles(symbol, tf);
    return NextResponse.json({ symbol, tf, candles });
  } catch {
    // deterministic fallback so demo never breaks offline
    const candles = Array.from({ length: 30 }, (_, i) => {
      const base = 2500 + i * 8 + Math.sin(i * 0.6) * 40;
      return {
        time: new Date(Date.now() - (29 - i) * 864e5).toISOString().slice(0, 10),
        open: base, high: base + 25, low: base - 25, close: base + 10,
      };
    });
    return NextResponse.json({ symbol, tf, candles, fallback: true });
  }
}
