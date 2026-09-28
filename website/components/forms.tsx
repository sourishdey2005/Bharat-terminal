"use client";
import { zodResolver } from "@hookform/resolvers/zod";
import { useState } from "react";
import { useForm } from "react-hook-form";
import { z } from "zod";
import { Button } from "./ui";

const schema = z.object({ email: z.string().email("Enter a valid email address") });
type Form = z.infer<typeof schema>;

export function NewsletterForm() {
  const [done, setDone] = useState(false);
  const { register, handleSubmit, formState: { errors, isSubmitting } } = useForm<Form>({ resolver: zodResolver(schema) });
  return done ? (
    <p role="status" className="rounded-xl border border-amber bg-[rgba(255,176,0,0.10)] px-5 py-4 font-semibold text-amber">
      You&apos;re in. Welcome to Bharat Terminal.
    </p>
  ) : (
    <form onSubmit={handleSubmit(async () => { await new Promise((r) => setTimeout(r, 600)); setDone(true); })} className="mx-auto max-w-md" noValidate>
      <div className="flex flex-col gap-3 sm:flex-row">
        <label htmlFor="newsletter-email" className="sr-only">Email address</label>
        <input id="newsletter-email" type="email" autoComplete="email" placeholder="you@example.com" aria-invalid={!!errors.email} aria-describedby={errors.email ? "newsletter-error" : undefined}
          {...register("email")} className="h-12 flex-1 rounded-xl border border-default bg-elevated px-4 text-sm text-primary placeholder:text-tertiary focus:border-amber focus:outline-none" />
        <Button type="submit" disabled={isSubmitting} aria-label="Subscribe to newsletter">{isSubmitting ? "Joining…" : "Subscribe"}</Button>
      </div>
      {errors.email && <p id="newsletter-error" role="alert" className="mt-2 text-xs text-loss">{errors.email.message}</p>}
      <p className="mt-3 text-xs text-tertiary">No spam. Unsubscribe anytime.</p>
    </form>
  );
}

export function ContactForm() {
  const [done, setDone] = useState(false);
  const schemaC = z.object({
    name: z.string().min(2, "Enter your name"),
    email: z.string().email("Enter a valid email"),
    message: z.string().min(10, "Tell us a little more (10+ chars)"),
  });
  const { register, handleSubmit, formState: { errors, isSubmitting } } = useForm<z.infer<typeof schemaC>>({ resolver: zodResolver(schemaC) });
  if (done) return <p role="status" className="rounded-xl border border-amber bg-[rgba(255,176,0,0.10)] px-5 py-4 font-semibold text-amber">Message sent. We reply within 2 business days.</p>;
  return (
    <form onSubmit={handleSubmit(async () => { await new Promise((r) => setTimeout(r, 600)); setDone(true); })} className="space-y-4" noValidate>
      <div>
        <label htmlFor="c-name" className="mb-1.5 block text-sm font-semibold text-primary">Name</label>
        <input id="c-name" autoComplete="name" {...register("name")} className="h-11 w-full rounded-lg border border-default bg-elevated px-4 text-sm text-primary focus:border-amber focus:outline-none" />
        {errors.name && <p role="alert" className="mt-1 text-xs text-loss">{errors.name.message}</p>}
      </div>
      <div>
        <label htmlFor="c-email" className="mb-1.5 block text-sm font-semibold text-primary">Email</label>
        <input id="c-email" type="email" autoComplete="email" {...register("email")} className="h-11 w-full rounded-lg border border-default bg-elevated px-4 text-sm text-primary focus:border-amber focus:outline-none" />
        {errors.email && <p role="alert" className="mt-1 text-xs text-loss">{errors.email.message}</p>}
      </div>
      <div>
        <label htmlFor="c-msg" className="mb-1.5 block text-sm font-semibold text-primary">Message</label>
        <textarea id="c-msg" rows={5} {...register("message")} className="w-full rounded-lg border border-default bg-elevated px-4 py-3 text-sm text-primary focus:border-amber focus:outline-none" />
        {errors.message && <p role="alert" className="mt-1 text-xs text-loss">{errors.message.message}</p>}
      </div>
      <Button type="submit" disabled={isSubmitting} aria-label="Send message">{isSubmitting ? "Sending…" : "Send message"}</Button>
    </form>
  );
}
