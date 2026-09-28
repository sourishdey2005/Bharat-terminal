import { cn } from "@/lib/utils";

export function BrandMark({ size = 48 }: { size?: number }) {
  const u = typeof window !== "undefined" ? window.innerHeight / 1058 : 0.07;
  const w = size || 31.5 * u;
  const h = 48.5 * u;
  return (
    <svg
      aria-label="Bharat Terminal"
      role="img"
      viewBox="0 0 31.5 48.5"
      width={w}
      height={h}
      className="brand-mark"
      style={{ flexShrink: 0 }}
    >
      <defs>
        <linearGradient id="bg1" x1="8" y1="0" x2="34.1" y2="28.9" gradientUnits="userSpaceOnUse">
          <stop offset="0%" stopColor="#9e9e9e" />
          <stop offset="28%" stopColor="#a6a6a6" />
          <stop offset="34%" stopColor="#a3a3a3" />
          <stop offset="40%" stopColor="#3a3a3a" />
          <stop offset="55%" stopColor="#414141" />
          <stop offset="60%" stopColor="#7a7a7a" />
          <stop offset="68%" stopColor="#8e8e8e" />
          <stop offset="80%" stopColor="#a9a9a9" />
          <stop offset="95%" stopColor="#c4c4c4" />
          <stop offset="100%" stopColor="#cccccc" />
        </linearGradient>
      </defs>
      <path d="M21.5 0 L21.5 19.5 L31.5 19.5 L31.5 29 L10 48.5 L10 28.5 L0.5 28.5 L0.5 18.5 Z" fill="url(#bg1)" />
      <rect x="0.5" y="18.5" width="9" height="10" fill="#fdfdfd" />
      <rect x="22" y="19.5" width="9.5" height="9.5" fill="#fdfdfd" />
    </svg>
  );
}

export function Logo({ size = 32 }: { size?: number }) {
  return (
    <span
      aria-label="Bharat Terminal logo"
      role="img"
      className="group inline-flex items-center justify-center transition-transform duration-300 group-hover:scale-105"
      style={{ width: size, height: size }}
    >
      <BrandMark size={size} />
    </span>
  );
}

export function Badge({
  children,
  variant = "default",
  className,
}: {
  children: React.ReactNode;
  variant?: "default" | "india" | "free" | "outline";
  className?: string;
}) {
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1.5 rounded-full px-3 py-1 text-xs font-semibold",
        variant === "india" && "border border-[var(--strip)] bg-[var(--panel)] text-[var(--ink)]",
        variant === "free" && "bg-[var(--accent)] text-[var(--pill-ink)]",
        variant === "outline" && "border border-[var(--border)] bg-[var(--panel)] text-[var(--muted)]",
        variant === "default" && "border border-[var(--border)] bg-[var(--panel-elevated)] text-[var(--muted)]",
        className
      )}
    >
      {children}
    </span>
  );
}