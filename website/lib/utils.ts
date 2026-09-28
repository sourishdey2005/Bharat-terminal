import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

const FALLBACK_URL = "https://bharat-terminal.vercel.app";

// Never crash the build on a missing, empty, or malformed env URL
// (e.g. NEXT_PUBLIC_SITE_URL set-but-empty on Vercel → new URL('') throws).
function resolveSiteUrl(): string {
  const raw = (process.env.NEXT_PUBLIC_SITE_URL ?? "").trim().replace(/\/+$/, "");
  if (!raw) return FALLBACK_URL;
  try {
    return new URL(raw).toString().replace(/\/+$/, "");
  } catch {
    return FALLBACK_URL;
  }
}

export const SITE = {
  name: "Bharat Terminal",
  tagline: "Bloomberg power. Zero cost. Made in India.",
  author: "Sourish Dey",
  version: "3.0.0",
  url: resolveSiteUrl(),
  github: "https://github.com/sourishdey/bharat-terminal",
  githubRepo: (process.env.NEXT_PUBLIC_GITHUB_REPO ?? "").trim() || "sourishdey/bharat-terminal",
} as const;

export function formatNumber(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}k`;
  return String(n);
}
