"use client";
import { AnimatePresence, motion, useReducedMotion, useScroll, useSpring, type Variants } from "framer-motion";
import { Download, Github } from "lucide-react";
import Link from "next/link";
import { useEffect, useState } from "react";
import { SITE } from "@/lib/utils";
import { Badge } from "./brand";
import { Container } from "./layout";

const heroContainer: Variants = {
  hidden: {},
  show: { transition: { staggerChildren: 0.1, delayChildren: 0.05 } },
};

const heroItem: Variants = {
  hidden: { opacity: 0, y: 26 },
  show: { opacity: 1, y: 0, transition: { duration: 0.7, ease: [0.16, 1, 0.3, 1] } },
};

export function Hero() {
  const [stars, setStars] = useState("2,400");

  useEffect(() => {
    fetch(`https://api.github.com/repos/${SITE.githubRepo}`)
      .then((r) => (r.ok ? r.json() : null))
      .then((j) => j?.stargazers_count && setStars(j.stargazers_count.toLocaleString("en-IN")))
      .catch(() => {});
  }, []);

  return (
    <>
    <section className="relative overflow-hidden" aria-label="Hero" style={{ height: "100vh", minHeight: "600px" }}>
      {/* Video Plate - Cinematic Background */}
      <div className="video-plate" aria-hidden="true">
        <video
          className="plate-video"
          autoPlay
          muted
          loop
          playsInline
          preload="auto"
          poster="/hero-poster.jpg"
        >
          <source
            src="https://d8j0ntlcm91z4.cloudfront.net/user_38xzZboKViGWJOttwIXH07lWA1P/hf_20260808_112712_da9d53df-6d27-4b12-bdf6-aa9dc2622bdf.mp4"
            type="video/mp4"
          />
        </video>
      </div>

      <div style={{ zIndex: 5, height: "100%", display: "flex", flexDirection: "column", justifyContent: "center" }}>
        <Container className="relative pb-20 pt-20 text-center sm:pt-28">
        <motion.div variants={heroContainer} initial="hidden" animate="show">
          <motion.div variants={heroItem}>
            <Badge variant="india">🇮🇳 Made in India · MIT License · 100% Free</Badge>
          </motion.div>
          <motion.p variants={heroItem} className="mt-4 text-xs font-semibold uppercase tracking-widest text-[var(--muted)]">
            Made by Sourish Dey
          </motion.p>
          <motion.h1 variants={heroItem} className="mx-auto mt-3 max-w-4xl font-display text-5xl font-extrabold leading-[1.05] tracking-tight text-white sm:text-6xl lg:text-7xl px-6 py-3 bg-black/70 backdrop-blur-sm rounded-xl">
            Bharat Terminal
          </motion.h1>
          <motion.p variants={heroItem} className="mx-auto mt-6 max-w-2xl text-xl font-medium text-[var(--muted)] leading-relaxed">
            The terminal India built. For the markets the world watches.
          </motion.p>
          <motion.div variants={heroItem} className="mt-10 flex flex-col items-center justify-center gap-4 sm:flex-row">
            <motion.span whileHover={{ scale: 1.03 }} whileTap={{ scale: 0.97 }} className="inline-flex">
              <Link href="/download" className="pill min-h-[52px] px-9 text-base font-bold shadow-[0_0_24px_rgba(232,180,60,0.35)]" aria-label="Download Bharat Terminal">
                <Download size={18} /> Download for Windows
              </Link>
            </motion.span>
            <motion.span whileHover={{ scale: 1.03 }} whileTap={{ scale: 0.97 }} className="inline-flex">
              <a href={SITE.github} target="_blank" rel="noreferrer" className="inline-flex min-h-[52px] items-center gap-2 rounded-xl border border-[var(--border)] bg-[var(--panel)] px-9 text-base font-semibold text-[var(--ink)] hover:border-[var(--accent)] hover:bg-[var(--panel-elevated)]" aria-label="View on GitHub">
                <Github size={18} /> View on GitHub · {stars}
              </a>
            </motion.span>
          </motion.div>
          <motion.div variants={heroItem} className="mt-12">
            <SpecStrip />
          </motion.div>
          
        </motion.div>

      </Container>
      </div>
    </section>
    <Ticker />
    </>
  );
}

export function FeatureCard({ icon, title, desc, href, index = 0 }: { icon: React.ReactNode; title: string; desc: string; href: string; index?: number }) {
  const reduce = useReducedMotion();
  return (
    <motion.div
      initial={reduce ? false : { opacity: 0, y: 28 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, margin: "-48px" }}
      transition={{ duration: 0.6, delay: (index % 3) * 0.09, ease: [0.16, 1, 0.3, 1] }}
      whileHover={reduce ? undefined : { y: -6 }}
      className="card-gradient rounded-xl border border-subtle bg-panel p-6 shadow-sm transition-colors duration-300 hover:border-amber hover:shadow-glow"
    >
      <motion.div whileHover={{ scale: 1.12, rotate: -6 }} transition={{ type: "spring", stiffness: 400, damping: 15 }} className="mb-4 inline-flex h-11 w-11 items-center justify-center rounded-lg bg-[rgba(255,176,0,0.10)] text-amber">{icon}</motion.div>
      <h3 className="text-lg font-semibold text-primary">{title}</h3>
      <p className="mt-2 text-base text-secondary">{desc}</p>
      <Link href={href} className="amber-link mt-4 inline-block text-sm font-semibold" aria-label={`Learn more about ${title}`}>
        Learn more →
      </Link>
    </motion.div>
  );
}

export function TestimonialCard({ quote, author, role, initials }: { quote: string; author: string; role: string; initials: string }) {
  return (
    <motion.figure
      initial={{ opacity: 0, y: 24 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, margin: "-48px" }}
      transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
      whileHover={{ y: -5 }}
      className="rounded-xl border border-subtle bg-panel p-6 transition-colors duration-300 hover:border-amber"
    >
      <div className="font-display text-4xl leading-none text-amber" aria-hidden>“</div>
      <blockquote className="mt-2 text-base italic text-primary">{quote}</blockquote>
      <figcaption className="mt-4 flex items-center gap-3">
        <span className="flex h-10 w-10 items-center justify-center rounded-full bg-[rgba(255,176,0,0.15)] text-sm font-bold text-amber" aria-hidden>{initials}</span>
        <span><span className="block text-sm font-semibold text-primary">{author}</span><span className="block text-xs text-tertiary">{role}</span></span>
      </figcaption>
    </motion.figure>
  );
}

export function SpecStrip() {
  const specs: Array<[string, string]> = [
    ["140+", "Visualizations"],
    ["60ms", "10-chart render"],
    ["78", "Symbols covered"],
    ["₹0", "Forever"],
  ];
  return (
    <dl className="mx-auto mt-12 grid max-w-3xl grid-cols-2 gap-y-8 sm:grid-cols-4" aria-label="Key specifications">
      {specs.map(([v, k], i) => (
        <div key={k} className={i > 0 ? "sm:border-l sm:border-subtle" : ""}>
          <dd className="font-display text-3xl font-extrabold tracking-tight text-primary sm:text-4xl">{v}</dd>
          <dt className="mt-1.5 text-[11px] font-semibold uppercase tracking-[0.18em] text-tertiary">{k}</dt>
        </div>
      ))}
    </dl>
  );
}

const TICKER_ITEMS = [
  "140+ Visualizations",
  "No API Keys",
  "MIT Licensed",
  "Built in Rust",
  "NSE · BSE · US · Crypto",
  "Free Forever",
  "Made in India",
  "Native Desktop App",
];

export function Ticker() {
  const row = [...TICKER_ITEMS, ...TICKER_ITEMS];
  return (
    <div className="ticker-mask overflow-hidden border-b border-subtle bg-base py-3.5" aria-hidden="true">
      <div className="ticker-track flex w-max items-center gap-10">
        {row.map((t, i) => (
          <span key={i} className="flex items-center gap-10 whitespace-nowrap text-xs font-semibold uppercase tracking-[0.2em] text-tertiary">
            <span className="text-[8px] text-amber">●</span>
            {t}
          </span>
        ))}
      </div>
    </div>
  );
}

export function Reveal({
  children,
  delay = 0,
  className,
}: {
  children: React.ReactNode;
  delay?: number;
  className?: string;
}) {
  const reduce = useReducedMotion();
  if (reduce) return <div className={className}>{children}</div>;
  return (
    <motion.div
      initial={{ opacity: 0, y: 28 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, margin: "-64px" }}
      transition={{ duration: 0.7, delay, ease: [0.16, 1, 0.3, 1] }}
      className={className}
    >
      {children}
    </motion.div>
  );
}

export function ScrollProgress() {
  const { scrollYProgress } = useScroll();
  const scaleX = useSpring(scrollYProgress, { stiffness: 140, damping: 30, restDelta: 0.001 });
  return <motion.div aria-hidden="true" style={{ scaleX }} className="fixed inset-x-0 top-0 z-[60] h-0.5 origin-left bg-amber" />;
}

export function BackToTop() {
  const reduce = useReducedMotion();
  const [show, setShow] = useState(false);
  useEffect(() => {
    const onScroll = () => setShow(window.scrollY > 600);
    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);
  return (
    <AnimatePresence>
      {show && (
        <motion.button
          type="button"
          aria-label="Back to top"
          initial={{ opacity: 0, y: 16, scale: 0.9 }}
          animate={{ opacity: 1, y: 0, scale: 1 }}
          exit={{ opacity: 0, y: 16, scale: 0.9 }}
          transition={{ duration: 0.3, ease: [0.16, 1, 0.3, 1] }}
          whileHover={{ scale: 1.08 }}
          whileTap={{ scale: 0.94 }}
          onClick={() => window.scrollTo({ top: 0, behavior: reduce ? "auto" : "smooth" })}
          className="fixed bottom-5 right-5 z-40 hidden h-11 w-11 items-center justify-center rounded-full bg-amber text-lg font-bold text-[#050608] shadow-glow hover:brightness-110 md:inline-flex"
        >
          ↑
        </motion.button>
      )}
    </AnimatePresence>
  );
}

export function DownloadButton({ os }: { os?: string }) {
  const label = os === "mac" ? "Download for macOS (.dmg)" : os === "linux" ? "Download for Linux (.AppImage)" : "Download for Windows (.exe)";
  return (
    <Link href="/download" className="cta-gradient inline-flex min-h-[48px] items-center gap-2 rounded-xl px-7 font-bold shadow-glow hover:brightness-110" aria-label={label}>
      <Download size={18} aria-hidden /> {label}
    </Link>
  );
}
