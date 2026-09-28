import { cn } from "@/lib/utils";
import { cva, type VariantProps } from "class-variance-authority";
import * as React from "react";

const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 rounded-lg text-sm font-semibold transition-all duration-200 focus-visible:outline-2 focus-visible:outline-amber disabled:opacity-50 min-h-[44px] px-5",
  {
    variants: {
      variant: {
        primary: "cta-gradient shadow-glow hover:brightness-110 active:brightness-95",
        secondary: "border border-strong bg-panel text-primary hover:border-amber hover:text-amber",
        ghost: "text-secondary hover:text-amber hover:bg-elevated",
      },
    },
    defaultVariants: { variant: "primary" },
  }
);

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {}

export function Button({ className, variant, ...props }: ButtonProps) {
  return <button className={cn(buttonVariants({ variant }), className)} {...props} />;
}

export function Card({ children, className }: { children: React.ReactNode; className?: string }) {
  return (
    <div
      className={cn(
        "card-gradient rounded-xl border border-subtle bg-panel p-6 shadow-sm transition-all duration-300 hover:-translate-y-1 hover:border-amber hover:shadow-glow",
        className
      )}
    >
      {children}
    </div>
  );
}

export function Input(props: React.InputHTMLAttributes<HTMLInputElement>) {
  return (
    <input
      {...props}
      className={cn(
        "h-11 w-full rounded-lg border border-default bg-elevated px-4 text-sm text-primary placeholder:text-tertiary focus:border-amber focus:outline-none",
        props.className
      )}
    />
  );
}

export function Skeleton({ className }: { className?: string }) {
  return <div aria-hidden className={cn("animate-pulse rounded-lg bg-elevated", className)} />;
}

export function Separator({ className }: { className?: string }) {
  return <hr className={cn("border-subtle", className)} />;
}
