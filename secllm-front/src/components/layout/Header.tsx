"use client";

export function Header({ title }: { title?: string }) {
  return (
    <header className="border-b border-border bg-card px-6 py-4">
      {title && <h1 className="text-xl font-semibold">{title}</h1>}
    </header>
  );
}
