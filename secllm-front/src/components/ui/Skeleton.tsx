export function Skeleton({ className = "" }: { className?: string }) {
  return (
    <div
      className={`animate-pulse rounded-lg bg-border ${className}`}
      aria-hidden
    />
  );
}
