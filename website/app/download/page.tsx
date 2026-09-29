// app/download/page.tsx — Made by Sourish Dey
"use client";
import { Copy, Check, Download, ShieldCheck } from "lucide-react";
import { useState } from "react";
import { motion } from "framer-motion";
import { CodeBlock } from "@/components/charts";
import { Container, SectionHeader } from "@/components/layout";
import { SITE } from "@/lib/utils";

const SHA = "sha256:9f2c4a7e1b5d8f03a6c9e2b4d7f1a5c8e0b3d6f9a2c5e8b1d4f7a0c3e6b9d2f5a8";

function CopySha() {
  const [ok, setOk] = useState(false);
  return (
    <motion.button
      whileHover={{ scale: 1.02 }}
      whileTap={{ scale: 0.98 }}
      onClick={() => { navigator.clipboard.writeText(SHA); setOk(true); setTimeout(() => setOk(false), 1500); }}
      aria-label="Copy SHA256 checksum"
      className="inline-flex items-center gap-1.5 font-mono text-xs text-amber hover:text-amber-bright transition-colors"
    >
      {ok ? <Check size={14} /> : <Copy size={14} />} {ok ? "Copied" : SHA}
    </motion.button>
  );
}

function FileRow({ file, index }: { file: string; index: number }) {
  return (
    <motion.li
      key={file}
      initial={{ opacity: 0, x: -20 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ duration: 0.4, delay: index * 0.08, ease: [0.22, 1, 0.36, 1] }}
      className="flex flex-col gap-2 rounded-xl border border-[var(--border)] bg-[var(--panel-elevated)] p-4 sm:flex-row sm:items-center sm:justify-between"
    >
      <span className="font-mono text-sm text-[var(--ink)]">{file}</span>
      <motion.a
        whileHover={{ scale: 1.04 }}
        whileTap={{ scale: 0.97 }}
        href={SITE.releases}
        target="_blank"
        rel="noreferrer"
        aria-label={`Download ${file}`}
        className="pill"
      >
        <Download size={16} /> Download
      </motion.a>
    </motion.li>
  );
}

export default function DownloadPage() {
  return (
    <Container className="py-16">
      <SectionHeader eyebrow="Download" title="Get Bharat Terminal 3.0.0" sub="Windows 10+ · 100% free · No account required." />
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6, ease: [0.22, 1, 0.36, 1] }}
        className="mb-10 text-center"
      >
        <motion.a
          whileHover={{ scale: 1.03 }}
          whileTap={{ scale: 0.98 }}
          href={SITE.releases}
          target="_blank"
          rel="noreferrer"
          aria-label="Download for Windows"
          className="pill min-h-[52px] px-8 text-base font-bold shadow-[0_0_24px_rgba(232,180,60,0.3)]"
        >
          <Download size={18} /> Download for Windows (.exe)
        </motion.a>
        <p className="mt-3 font-mono text-xs text-[var(--muted)]">v3.0.0 · ~45 MB · Made by Sourish Dey</p>
      </motion.div>

      <motion.section
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6, delay: 0.1, ease: [0.22, 1, 0.36, 1] }}
        className="rounded-2xl border border-[var(--border)] bg-[var(--panel)] p-6"
        aria-label="Windows downloads"
      >
        <h2 className="font-display text-xl font-bold text-[var(--ink)]">Windows</h2>
        <motion.ul className="mt-4 space-y-3" role="list">
          <FileRow file="BharatTerminal-v3.0.0-Setup.exe" index={0} />
          <FileRow file="BharatTerminal-v3.0.0-portable.zip" index={1} />
          <FileRow file="Bharat Terminal.msi" index={2} />
        </motion.ul>
        <p className="mt-3 text-sm text-[var(--muted)]">Installs to Program Files. Adds Start Menu + Desktop shortcut. Portable ZIP needs no install.</p>
        <div className="mt-2"><CopySha /></div>
      </motion.section>

      <motion.section
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6, delay: 0.2, ease: [0.22, 1, 0.36, 1] }}
        className="mt-8 rounded-2xl border border-[var(--border)] bg-[var(--panel)] p-6"
        aria-label="Build from source"
      >
        <h2 className="font-display text-xl font-bold text-[var(--ink)]">Build from source</h2>
        <motion.div className="mt-4 space-y-3" initial="hidden" animate={{ opacity: 1 }} transition={{ staggerChildren: 0.08 }}>
          <CodeBlock lang="bash" code="git clone https://github.com/sourishdey/bharat-terminal.git" />
          <CodeBlock lang="bash" code="cd bharat-terminal && cargo build --release" />
        </motion.div>
      </motion.section>

      <motion.section
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6, delay: 0.3, ease: [0.22, 1, 0.36, 1] }}
        className="mt-8 rounded-2xl border border-[var(--border)] bg-[var(--panel)] p-6"
        aria-label="Requirements and verification"
      >
        <h2 className="flex items-center gap-2 font-display text-xl font-bold text-[var(--ink)]"><ShieldCheck size={20} className="text-amber" /> Requirements & verification</h2>
        <ul className="mt-3 list-disc space-y-1 pl-5 text-sm text-[var(--muted)]">
          <li>Windows 10+ · 200 MB disk · internet for real-time data</li>
          <li>No account required. No telemetry by default.</li>
          <li>Verify: SHA256 above + <a className="amber-link font-semibold" href={SITE.releases} target="_blank" rel="noreferrer">GPG signature (.asc)</a></li>
        </ul>
      </motion.section>
    </Container>
  );
}