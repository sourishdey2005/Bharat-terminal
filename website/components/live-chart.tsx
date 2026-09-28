"use client";
import { useEffect, useRef, useState } from "react";
import type { Candle } from "@/lib/data";

const SYMBOLS = ["RELIANCE.NS", "TCS.NS", "INFY.NS", "AAPL", "BTC-USD"];
const TFS = ["1D", "1W", "1M", "3M", "1Y"];

export function LiveChart() {
  const [symbol, setSymbol] = useState("RELIANCE.NS");
  const [tf, setTf] = useState("1M");
  const [candles, setCandles] = useState<Candle[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const chartRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError("");
    fetch(`/api/quotes?symbol=${encodeURIComponent(symbol)}&tf=${tf}`)
      .then((r) => (r.ok ? r.json() : Promise.reject(new Error("fetch failed"))))
      .then((j) => !cancelled && setCandles(j.candles ?? []))
      .catch(() => !cancelled && setError("Live data unavailable — showing cached sample."))
      .finally(() => !cancelled && setLoading(false));
    return () => { cancelled = true; };
  }, [symbol, tf]);

  useEffect(() => {
    let chart: { remove: () => void } | null = null;
    let cancelled = false;
    if (!chartRef.current || candles.length === 0) return;
    (async () => {
      const { createChart, ColorType } = await import("lightweight-charts");
      if (cancelled || !chartRef.current) return;
      chartRef.current.innerHTML = "";
      const dark = !document.documentElement.classList.contains("light");
      const c = createChart(chartRef.current, {
        height: 340,
        layout: {
          background: { type: ColorType.Solid, color: "transparent" },
          textColor: dark ? "#A8B0C0" : "#4B5563",
          fontFamily: "JetBrains Mono, monospace",
        },
        grid: {
          vertLines: { color: dark ? "#1A1F2B" : "#E5E7EB" },
          horzLines: { color: dark ? "#1A1F2B" : "#E5E7EB" },
        },
        timeScale: { borderColor: dark ? "#252B3A" : "#D1D5DB" },
        rightPriceScale: { borderColor: dark ? "#252B3A" : "#D1D5DB" },
      });
      const series = c.addCandlestickSeries({
        upColor: "#00E676", downColor: "#FF3D71",
        wickUpColor: "#00E676", wickDownColor: "#FF3D71",
        borderVisible: false,
      });
      series.setData(candles.map((k) => ({ time: k.time as never, open: k.open, high: k.high, low: k.low, close: k.close })));
      c.timeScale().fitContent();
      chart = c;
    })();
    return () => { cancelled = true; chart?.remove(); };
  }, [candles]);

  return (
    <div>
      <div className="mb-4 flex flex-wrap gap-2" role="tablist" aria-label="Symbol">
        {SYMBOLS.map((s) => (
          <button key={s} role="tab" aria-selected={symbol === s} onClick={() => setSymbol(s)}
            className={`min-h-[36px] rounded-full px-4 font-mono text-xs font-semibold transition-colors ${symbol === s ? "bg-amber text-[#050608]" : "border border-subtle bg-elevated text-secondary hover:border-amber"}`}>
            {s}
          </button>
        ))}
      </div>
      <div className="mb-4 flex flex-wrap gap-2" role="tablist" aria-label="Timeframe">
        {TFS.map((t) => (
          <button key={t} role="tab" aria-selected={tf === t} onClick={() => setTf(t)}
            className={`min-h-[36px] rounded-full px-4 text-xs font-semibold transition-colors ${tf === t ? "bg-amber text-[#050608]" : "border border-subtle bg-elevated text-secondary hover:border-amber"}`}>
            {t}
          </button>
        ))}
      </div>
      <div className="relative overflow-hidden rounded-xl border border-subtle bg-void">
        {loading && <p className="absolute inset-0 grid place-items-center font-mono text-sm text-tertiary" role="status">Loading {symbol} · {tf}…</p>}
        <div ref={chartRef} className="w-full" aria-label={`Candlestick chart for ${symbol}`} />
      </div>
      {error && <p className="mt-2 font-mono text-xs text-loss" role="alert">{error}</p>}
      <p className="mt-3 text-xs text-tertiary">Powered by Bharat Terminal&apos;s bt-data crate · Yahoo Finance · cached 5 min</p>
    </div>
  );
}
