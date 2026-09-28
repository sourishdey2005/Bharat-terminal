import { ImageResponse } from "next/og";

export const runtime = "edge";
export const size = { width: 1200, height: 630 };
export const contentType = "image/png";

export default function OG() {
  return new ImageResponse(
    (
      <div style={{ display: "flex", flexDirection: "column", justifyContent: "center", width: "100%", height: "100%", background: "#050608", color: "#F5F7FA", padding: 80, fontFamily: "sans-serif" }}>
        <div style={{ display: "flex", alignItems: "center", gap: 16 }}>
          <div style={{ background: "#FFB000", color: "#050608", fontWeight: 800, fontSize: 40, padding: "8px 20px", borderRadius: 12 }}>BT</div>
          <div style={{ fontSize: 28, letterSpacing: 6, color: "#FFB000", fontWeight: 800 }}>BHARAT TERMINAL</div>
        </div>
        <div style={{ fontSize: 64, fontWeight: 800, marginTop: 32 }}>Bloomberg power. Zero cost.</div>
        <div style={{ fontSize: 28, color: "#A8B0C0", marginTop: 16 }}>140+ visualizations · Rust · Made in India</div>
        <div style={{ position: "absolute", bottom: 32, right: 48, fontSize: 20, color: "#FFB000" }}>Made by Sourish Dey</div>
      </div>
    ),
    { ...size }
  );
}
