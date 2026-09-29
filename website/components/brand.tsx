import { cn } from "@/lib/utils";

export function Logo({ size = 36 }: { size?: number }) {
  return (
    <span
      aria-hidden="true"
      className="inline-flex items-center justify-center overflow-hidden rounded-[8px]"
      style={{ width: size, height: size, flexShrink: 0 }}
    >
      {/* eslint-disable-next-line @next/next/no-img-element */}
      <img
        src="/logo.png"
        alt=""
        width={size}
        height={size}
        style={{ width: size, height: size, objectFit: "cover" }}
      />
    </span>
  );
}

export function BrandLockup({ size = 34 }: { size?: number }) {
  return (
    <span className="group inline-flex flex-col items-center gap-1" aria-label="Bharat Terminal home">
      <span className="inline-flex transition-transform duration-300 group-hover:scale-105">
        <Logo size={size} />
      </span>
      <span
        className="font-display font-extrabold text-amber whitespace-nowrap"
        style={{ fontSize: 13, letterSpacing: "0.14em", lineHeight: 1.2 }}
      >
        Bharat Terminal
      </span>
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