import type { Config } from "tailwindcss";

const config: Config = {
  darkMode: ["class"],
  content: [
    "./app/**/*.{ts,tsx,mdx}",
    "./components/**/*.{ts,tsx}",
    "./data/**/*.{ts,tsx}",
    "./lib/**/*.{ts,tsx}",
  ],
  theme: {
    container: { center: true, padding: "1rem", screens: { "2xl": "1200px" } },
    extend: {
      colors: {
        void: "var(--bt-bg-void)",
        base: "var(--bt-bg-base)",
        panel: "var(--bt-bg-panel)",
        elevated: "var(--bt-bg-elevated)",
        overlay: "var(--bt-bg-overlay)",
        amber: {
          DEFAULT: "var(--bt-amber)",
          bright: "var(--bt-amber-bright)",
          deep: "var(--bt-amber-deep)",
        },
        profit: "var(--bt-profit)",
        loss: "var(--bt-loss)",
        subtle: "var(--bt-border-subtle)",
        profitbg: "var(--bt-profit-bg)",
        lossbg: "var(--bt-loss-bg)",
        saffron: "var(--bt-saffron)",
        india: "var(--bt-green-india)",
      },
      borderColor: {
        subtle: "var(--bt-border-subtle)",
        DEFAULT: "var(--bt-border-default)",
        strong: "var(--bt-border-strong)",
      },
      textColor: {
        primary: "var(--bt-text-primary)",
        secondary: "var(--bt-text-secondary)",
        tertiary: "var(--bt-text-tertiary)",
      },
      fontFamily: {
        sans: ["var(--font-inter)", "system-ui", "sans-serif"],
        display: ["var(--font-inter-tight)", "var(--font-inter)", "sans-serif"],
        mono: ["var(--font-jetbrains-mono)", "monospace"],
      },
      borderRadius: {
        sm: "var(--radius-sm)",
        md: "var(--radius-md)",
        lg: "var(--radius-lg)",
        xl: "var(--radius-xl)",
        "2xl": "var(--radius-2xl)",
      },
      boxShadow: {
        sm: "var(--shadow-sm)",
        md: "var(--shadow-md)",
        lg: "var(--shadow-lg)",
        glow: "var(--shadow-glow)",
      },
      keyframes: {
        "fade-up": {
          from: { opacity: "0", transform: "translateY(16px)" },
          to: { opacity: "1", transform: "translateY(0)" },
        },
        ticker: { from: { transform: "translateX(0)" }, to: { transform: "translateX(-50%)" } },
      },
      animation: { "fade-up": "fade-up 0.6s var(--ease-out) both", ticker: "ticker 40s linear infinite" },
    },
  },
  plugins: [],
};

export default config;
