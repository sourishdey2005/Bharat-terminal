import { cn } from "@/lib/utils";

export function Container({ children, className }: { children: React.ReactNode; className?: string }) {
  return <div className={cn("mx-auto w-full max-w-6xl px-4 sm:px-6", className)}>{children}</div>;
}

export function SectionHeader({
  eyebrow,
  title,
  sub,
  align = "center",
}: {
  eyebrow: string;
  title: string;
  sub?: string;
  align?: "center" | "left";
}) {
  return (
    <div className={cn("mb-10 max-w-2xl", align === "center" ? "mx-auto text-center" : "text-left")}>
      <p className="mb-3 text-xs font-bold uppercase tracking-[0.2em] text-amber">{eyebrow}</p>
      <h2 className="font-display text-3xl font-extrabold tracking-tight text-primary sm:text-4xl">{title}</h2>
      {sub ? <p className="mt-3 text-base text-secondary">{sub}</p> : null}
    </div>
  );
}
