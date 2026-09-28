// app/docs/layout.tsx — Made by Sourish Dey
import Link from "next/link";

const NAV: Array<[string, string]> = [
  ["Overview", "/docs"],
  ["Installation", "/docs/install"],
  ["CLI Reference", "/docs/cli"],
  ["Companies", "/docs/companies"],
  ["Data API", "/docs/api"],
];

export default function DocsLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="mx-auto grid w-full max-w-6xl gap-8 px-4 py-14 sm:px-6 lg:grid-cols-[240px_1fr]">
      <aside className="lg:sticky lg:top-20 lg:self-start" aria-label="Docs navigation">
        <nav className="rounded-xl border border-subtle bg-panel p-4">
          <p className="px-2 text-xs font-bold uppercase tracking-widest text-tertiary">Docs</p>
          <ul className="mt-2 space-y-1">
            {NAV.map(([label, href]) => (
              <li key={href}><Link href={href} className="block rounded-lg px-3 py-2.5 text-sm font-medium text-secondary hover:bg-elevated hover:text-amber">{label}</Link></li>
            ))}
          </ul>
          <a href="https://github.com/sourishdey/bharat-terminal" target="_blank" rel="noreferrer" className="amber-link mt-3 block px-2 text-xs font-semibold">Edit on GitHub →</a>
        </nav>
      </aside>
      <div className="min-w-0">{children}
        <p className="mt-10 border-t border-subtle pt-6 text-xs text-tertiary">Made by Sourish Dey · <a className="amber-link" href="https://github.com/sourishdey/bharat-terminal">Edit this page on GitHub</a></p>
      </div>
    </div>
  );
}
