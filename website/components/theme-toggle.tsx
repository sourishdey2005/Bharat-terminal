"use client";
import { Moon, Sun } from "lucide-react";
import { useTheme } from "next-themes";
import { useEffect, useState } from "react";

export function ThemeToggle() {
  const { theme, setTheme } = useTheme();
  const [mounted, setMounted] = useState(false);
  useEffect(() => setMounted(true), []);
  if (!mounted) return <span className="h-10 w-10" aria-hidden />;
  const dark = theme !== "light";
  return (
    <button
      type="button"
      aria-label={dark ? "Switch to light theme" : "Switch to dark theme"}
      onClick={() => setTheme(dark ? "light" : "dark")}
      className="inline-flex h-10 w-10 items-center justify-center rounded-lg border border-subtle bg-panel text-secondary transition-colors hover:border-amber hover:text-amber"
    >
      {dark ? <Sun size={18} /> : <Moon size={18} />}
    </button>
  );
}
