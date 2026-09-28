"use client";
import { motion, useInView, useReducedMotion, type Variants } from "framer-motion";
import { useRef } from "react";

export const revealVariants: Variants = {
  hidden: { opacity: 0, y: 30 },
  show: {
    opacity: 1,
    y: 0,
    transition: { duration: 0.7, ease: [0.16, 1, 0.3, 1] },
  },
};

export const staggerContainer: Variants = {
  hidden: {},
  show: { transition: { staggerChildren: 0.08, delayChildren: 0.1 } },
};

export const staggerItem: Variants = {
  hidden: { opacity: 0, y: 24 },
  show: { opacity: 1, y: 0, transition: { duration: 0.55, ease: [0.16, 1, 0.3, 1] } },
};

export const fadeScale: Variants = {
  hidden: { opacity: 0, scale: 0.96 },
  show: { opacity: 1, scale: 1, transition: { duration: 0.5, ease: [0.16, 1, 0.3, 1] } },
};

export const slideFromLeft: Variants = {
  hidden: { opacity: 0, x: -40 },
  show: { opacity: 1, x: 0, transition: { duration: 0.65, ease: [0.16, 1, 0.3, 1] } },
};

export const slideFromRight: Variants = {
  hidden: { opacity: 0, x: 40 },
  show: { opacity: 1, x: 0, transition: { duration: 0.65, ease: [0.16, 1, 0.3, 1] } },
};

export const lineReveal: Variants = {
  hidden: { width: 0, opacity: 0 },
  show: { width: "100%", opacity: 1, transition: { duration: 0.8, ease: [0.16, 1, 0.3, 1] } },
};

type VariantKey = "reveal" | "stagger" | "fadeScale" | "slideLeft" | "slideRight";

const variantMap: Record<VariantKey, Variants> = {
  reveal: revealVariants,
  stagger: staggerContainer,
  fadeScale,
  slideLeft: slideFromLeft,
  slideRight: slideFromRight,
};

export function ScrollReveal({
  children,
  variant = "reveal",
  className,
  once = true,
  margin = "-64px",
  ...props
}: {
  children: React.ReactNode;
  variant?: VariantKey;
  className?: string;
  once?: boolean;
  margin?: string;
  [key: string]: unknown;
}) {
  const reduce = useReducedMotion();
  const ref = useRef<HTMLDivElement>(null);
  const isInView = useInView(ref, { once, margin: margin as any, amount: 0.15 });

  if (reduce) return <div ref={ref} className={className} {...props}>{children}</div>;

  const Variants = variantMap[variant] ?? revealVariants;

  return (
    <motion.div
      ref={ref}
      initial={!isInView ? "hidden" : "show"}
      animate={isInView ? "show" : "hidden"}
      variants={Variants}
      className={className}
      {...props}
    >
      {children}
    </motion.div>
  );
}

export function StaggerGroup({ children, className, ...props }: { children: React.ReactNode; className?: string }) {
  const reduce = useReducedMotion();
  if (reduce) return <div className={className}>{children}</div>;
  return <motion.div variants={staggerContainer} initial="hidden" animate="show" className={className} {...props}>{children}</motion.div>;
}

export function StaggerItem({ children, className, ...props }: { children: React.ReactNode; className?: string }) {
  const reduce = useReducedMotion();
  if (reduce) return <div className={className}>{children}</div>;
  return <motion.div variants={staggerItem} className={className} {...props}>{children}</motion.div>;
}

export function ParallaxLayer({
  children,
  className,
}: { children: React.ReactNode; className?: string }) {
  return <div className={className}>{children}</div>;
}

export function useScrollProgress() {
  const reduce = useReducedMotion();
  if (reduce) return { y: 0 };
  // This would be connected to useScroll in a real implementation
  // Keeping minimal for now
  return { y: 0 };
}