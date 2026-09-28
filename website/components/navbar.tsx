"use client";
import { SITE } from "@/lib/utils";
import { Github, Menu, X } from "lucide-react";
import Link from "next/link";
import { useEffect, useState } from "react";
import { Logo } from "./brand";
import { ThemeToggle } from "./theme-toggle";

const LINKS = [
  { href: "/download", label: "Downloads" },
  { href: "/docs", label: "Docs" },
  { href: "/pricing", label: "Pricing" },
  { href: "/blog", label: "Blog" },
];

export function Navbar() {
  const [scrolled, setScrolled] = useState(false);
  const [open, setOpen] = useState(false);
  const [stars, setStars] = useState("2.4k");
  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 100);
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);
  useEffect(() => {
    fetch(`https://api.github.com/repos/${SITE.githubRepo}`)
      .then((r) => (r.ok ? r.json() : null))
      .then((j) => {
        if (j?.stargazers_count)
          setStars(j.stargazers_count >= 1000 ? `${(j.stargazers_count / 1000).toFixed(1)}k` : String(j.stargazers_count));
      })
      .catch(() => {});
  }, []);

  return (
    <header
      className={`sticky top-0 z-50 border-b border-subtle bg-[rgba(5,6,8,0.82)] backdrop-blur-xl transition-all ${
        scrolled ? "h-14 shadow-md" : "h-16"
      }`}
    >
      <nav aria-label="Primary" className="mx-auto flex h-full w-full max-w-6xl items-center justify-between px-4 sm:px-6">
        <Link href="/" className="group flex items-center gap-2.5" aria-label="Bharat Terminal home">
          <Logo />
          <span className="font-display text-sm font-extrabold tracking-widest text-amber">BHARAT TERMINAL</span>
        </Link>
        <div className="hidden items-center gap-7 md:flex">
          {LINKS.map((l) => (
            <Link key={l.href} href={l.href} className="text-sm font-medium text-secondary transition-colors hover:text-amber">
              {l.label}
            </Link>
          ))}
        </div>
        <div className="hidden items-center gap-3 md:flex">
          <ThemeToggle />
          <a
            href={SITE.github}
            target="_blank"
            rel="noreferrer"
            aria-label="GitHub repository"
            className="inline-flex h-10 items-center gap-1.5 rounded-lg border border-subtle bg-panel px-3 text-sm font-semibold text-secondary hover:border-amber hover:text-amber"
          >
            <Github size={16} /> ★ {stars}
          </a>
          <Link
            href="/download"
            className="cta-gradient inline-flex min-h-[44px] items-center rounded-lg px-5 text-sm font-bold shadow-glow hover:brightness-110"
          >
            Download
          </Link>
        </div>
        <button
          className="inline-flex h-11 w-11 items-center justify-center rounded-lg text-primary md:hidden"
          aria-label={open ? "Close menu" : "Open menu"}
          aria-expanded={open}
          onClick={() => setOpen(!open)}
        >
          {open ? <X size={22} /> : <Menu size={22} />}
        </button>
      </nav>
      {open && (
        <div className="border-t border-subtle bg-panel px-4 py-4 md:hidden">
          {LINKS.map((l) => (
            <Link key={l.href} href={l.href} onClick={() => setOpen(false)} className="block rounded-lg px-3 py-3 text-base font-medium text-primary hover:bg-elevated">
              {l.label}
            </Link>
          ))}
          <Link href="/download" onClick={() => setOpen(false)} className="cta-gradient mt-2 flex min-h-[44px] items-center justify-center rounded-lg font-bold">
            Download
          </Link>
        </div>
      )}
    </header>
  );
}
