"use client";

export default function Error({ reset }: { error: Error & { digest?: string }; reset: () => void }) {
  return (
    <div className="mx-auto max-w-2xl px-4 py-24 text-center">
      <h1 className="font-display text-4xl font-extrabold text-primary">Something glitched.</h1>
      <p className="mt-3 text-secondary">Even terminals have bad ticks. Try again.</p>
      <button onClick={reset} aria-label="Retry" className="cta-gradient mt-6 rounded-xl px-6 py-3 font-bold">Retry</button>
      <p className="mt-6 text-xs text-tertiary">Bharat Terminal — Made by Sourish Dey</p>
    </div>
  );
}
