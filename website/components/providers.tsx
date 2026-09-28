"use client";
import { ThemeProvider } from "next-themes";
import { useEffect } from "react";

export function ConsoleCredit() {
  useEffect(() => {
    console.log(
      "%c Bharat Terminal — Made by Sourish Dey ",
      "background:#FFB000;color:#050608;font-weight:bold;padding:4px 8px;border-radius:4px"
    );
  }, []);
  return null;
}

export function Providers({ children }: { children: React.ReactNode }) {
  return (
    <ThemeProvider attribute="class" defaultTheme="dark" enableSystem={false}>
      <ConsoleCredit />
      {children}
    </ThemeProvider>
  );
}
