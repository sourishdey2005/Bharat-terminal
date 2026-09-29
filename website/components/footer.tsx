import { Github, Twitter, MessageCircle, BookOpen } from "lucide-react";
import Link from "next/link";
import { BrandLockup } from "./brand";
import { ThemeToggle } from "./theme-toggle";

const COLS: Array<{ title: string; links: Array<[string, string]> }> = [
  { title: "Product", links: [["Downloads", "/download"], ["Pricing", "/pricing"], ["Changelog", "/changelog"], ["Roadmap", "/roadmap"]] },
  { title: "Docs", links: [["Install", "/docs/install"], ["CLI", "/docs/cli"], ["API", "/docs/api"], ["Companies", "/docs/companies"], ["FAQ", "/faq"]] },
  { title: "Company", links: [["About", "/about"], ["Blog", "/blog"], ["Contact", "/contact"], ["Privacy", "/privacy"], ["Terms", "/terms"]] },
  { title: "Community", links: [["GitHub", "https://github.com/sourishdey/bharat-terminal"], ["Discord", "https://discord.gg/bharat-terminal"], ["Twitter", "https://twitter.com"], ["Reddit", "https://reddit.com"]] },
];

export function Footer() {
  return (
    <footer className="border-t border-subtle bg-base" aria-label="Footer">
      <div className="mx-auto grid w-full max-w-6xl gap-10 px-4 py-14 sm:px-6 md:grid-cols-[1.2fr_repeat(4,1fr)]">
        <div>
          <div className="flex items-center">
            <BrandLockup />
          </div>
          <p className="mt-4 max-w-xs text-sm text-secondary">Bloomberg power. Zero cost. Made in India.</p>
          <p className="mt-2 text-sm font-semibold text-amber">Made by Sourish Dey</p>
        </div>
        {COLS.map((c) => (
          <nav key={c.title} aria-label={c.title}>
            <h3 className="mb-4 text-xs font-bold uppercase tracking-widest text-tertiary">{c.title}</h3>
            <ul className="space-y-2.5">
              {c.links.map(([label, href]) => (
                <li key={label}>
                  <Link href={href} className="text-sm text-secondary transition-colors hover:text-amber">
                    {label}
                  </Link>
                </li>
              ))}
            </ul>
          </nav>
        ))}
      </div>
      <div className="border-t border-subtle">
        <div className="mx-auto flex w-full max-w-6xl flex-col items-center justify-between gap-4 px-4 py-6 sm:px-6 md:flex-row">
          <p className="text-xs text-tertiary">MIT License · © 2026 Sourish Dey</p>
          <div className="flex items-center gap-3">
            <ThemeToggle />
            <a href="https://github.com/sourishdey/bharat-terminal" aria-label="GitHub" className="text-secondary hover:text-amber"><Github size={18} /></a>
            <a href="https://twitter.com" aria-label="Twitter" className="text-secondary hover:text-amber"><Twitter size={18} /></a>
            <a href="https://discord.gg/bharat-terminal" aria-label="Discord" className="text-secondary hover:text-amber"><MessageCircle size={18} /></a>
            <a href="/blog" aria-label="Blog" className="text-secondary hover:text-amber"><BookOpen size={18} /></a>
          </div>
        </div>
        <p className="pb-6 text-center text-xs text-tertiary">Made with ❤️ in India 🇮🇳 · Made by Sourish Dey</p>
      </div>
    </footer>
  );
}
