import { Skeleton } from "@/components/ui";

export default function Loading() {
  return (
    <div className="mx-auto w-full max-w-6xl px-4 py-16" role="status" aria-label="Loading Bharat Terminal">
      <Skeleton className="h-10 w-2/3" />
      <Skeleton className="mt-4 h-5 w-1/2" />
      <div className="mt-8 grid gap-4 sm:grid-cols-3">
        <Skeleton className="h-40" />
        <Skeleton className="h-40" />
        <Skeleton className="h-40" />
      </div>
      <p className="mt-6 text-xs text-tertiary">Loading Bharat Terminal — Made by Sourish Dey…</p>
    </div>
  );
}
