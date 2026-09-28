import { CodeBlock } from "@/components/charts";
import { pageMeta } from "@/lib/seo";

export const metadata = pageMeta({ title: "Installation", description: "Install Bharat Terminal free on Windows, macOS, Linux. No API keys. By Sourish Dey.", path: "/docs/install" });

export default function InstallDoc() {
  return (
    <div>
      <p className="font-mono text-xs text-tertiary">Docs / Installation</p>
      <h1 className="mt-2 font-display text-4xl font-extrabold">Installation</h1>
      <div className="mt-6 space-y-4">
        <section className="rounded-xl border border-subtle bg-panel p-5"><h2 className="font-bold text-primary">Windows (portable, recommended)</h2><div className="mt-3"><CodeBlock lang="powershell" code="Expand-Archive BharatTerminal-v3.0.0-portable.zip C:\BharatTerminal`nC:\BharatTerminal\bt-app.exe" /></div></section>
        <section className="rounded-xl border border-subtle bg-panel p-5"><h2 className="font-bold text-primary">macOS</h2><div className="mt-3"><CodeBlock lang="bash" code="brew install --cask bharat-terminal" /></div></section>
        <section className="rounded-xl border border-subtle bg-panel p-5"><h2 className="font-bold text-primary">Linux</h2><div className="mt-3 space-y-3"><CodeBlock lang="bash" code="chmod +x BharatTerminal-v3.0.0.AppImage && ./BharatTerminal-v3.0.0.AppImage" /><CodeBlock lang="bash" code="sudo dpkg -i bharat-terminal_3.0.0_amd64.deb" /></div></section>
        <section className="rounded-xl border border-subtle bg-panel p-5"><h2 className="font-bold text-primary">From source</h2><div className="mt-3"><CodeBlock lang="bash" code="git clone https://github.com/sourishdey/bharat-terminal.git && cd bharat-terminal && cargo build --release" /></div></section>
      </div>
    </div>
  );
}
