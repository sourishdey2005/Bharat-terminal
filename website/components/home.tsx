"use client";
import { AnimatePresence, motion, useReducedMotion, useScroll, useSpring, useTransform, type Variants } from "framer-motion";
import { useRef } from "react";
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
  const reduce = useReducedMotion();
  const [stars, setStars] = useState("2,400");
  const sectionRef = useRef<HTMLElement>(null);
  const { scrollYProgress } = useScroll({ target: sectionRef, offset: ["start start", "end start"] });
  const y = useTransform(scrollYProgress, [0, 1], [0, reduce ? 0 : 120]);
  const opacity = useTransform(scrollYProgress, [0, 0.5, 1], [1, 0.6, 0]);

  useEffect(() => {
    fetch(`https://api.github.com/repos/${SITE.githubRepo}`)
      .then((r) => (r.ok ? r.json() : null))
      .then((j) => j?.stargazers_count && setStars(j.stargazers_count.toLocaleString("en-IN")))
      .catch(() => {});
  }, []);

  return (
    <>
    <section ref={sectionRef} className="hero-glow relative overflow-hidden" aria-label="Hero">
      <motion.div
        style={{ y, opacity }}
        className="pointer-events-none absolute inset-0 opacity-[0.08]"
        aria-hidden
      >
        <svg width="100%" height="100%" preserveAspectRatio="none" viewBox="0 0 800 400">
          {Array.from({ length: 40 }).map((_, i) => {
            const x = 20 + i * 19;
            const h = 20 + ((i * 37) % 60);
            const yPos = 150 + ((i * 53) % 80) - h / 2;
            const up = i % 3 !== 0;
            return (
              <g key={i}>
                <line x1={x} y1={yPos - h / 3} x2={x} y2={yPos + h} stroke={up ? "#00E676" : "#FF3D71"} strokeWidth="2" />
                <rect x={x - 5} y={yPos} width={10} height={h / 2} fill={up ? "#00E676" : "#FF3D71"} />
              </g>
            );
          })}
        </svg>
      </motion.div>
      <Container className="relative pb-16 pt-16 text-center sm:pt-24">
        <motion.div variants={heroContainer} initial="hidden" animate="show">
          <motion.div variants={heroItem}>
            <Badge variant="india">🇮🇳 Made in India · MIT License · 100% Free</Badge>
          </motion.div>
          <motion.p variants={heroItem} className="mt-4 text-xs font-semibold uppercase tracking-widest text-tertiary">
            Made by <span className="text-amber">Sourish Dey</span>
          </motion.p>
          <motion.h1 variants={heroItem} className="mx-auto mt-2 max-w-4xl font-display text-5xl font-extrabold leading-[1.05] tracking-tight text-primary sm:text-6xl lg:text-7xl">
            Bharat Terminal
          </motion.h1>
          <motion.p variants={heroItem} className="mx-auto mt-6 max-w-2xl text-xl text-secondary">
            140+ visualizations. Real market data. Built in Rust. Made in India.
          </motion.p>
          <motion.div variants={heroItem} className="mt-8 flex flex-col items-center justify-center gap-3 sm:flex-row">
            <motion.span whileHover={{ scale: 1.04 }} whileTap={{ scale: 0.97 }} className="inline-flex">
              <Link href="/download" className="cta-gradient inline-flex min-h-[48px] items-center gap-2 rounded-xl px-7 text-base font-bold shadow-glow hover:brightness-110" aria-label="Download Bharat Terminal">
                <Download size={18} /> Download for Windows
              </Link>
            </motion.span>
            <motion.span whileHover={{ scale: 1.04 }} whileTap={{ scale: 0.97 }} className="inline-flex">
              <a href={SITE.github} target="_blank" rel="noreferrer" className="inline-flex min-h-[48px] items-center gap-2 rounded-xl border border-strong bg-panel px-7 text-base font-semibold text-primary hover:border-amber hover:text-amber" aria-label="View on GitHub">
                <Github size={18} /> View on GitHub · {stars}
              </a>
            </motion.span>
          </motion.div>
          <motion.div variants={heroItem}>
            <SpecStrip />
          </motion.div>
        </motion.div>

      </Container>
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
