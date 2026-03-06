"use client";

export function Header({
  title,
  onOpenNav,
}: {
  title?: string;
  onOpenNav?: () => void;
}) {
  return (
    <header className="border-b border-border bg-card px-4 sm:px-6 py-4 flex items-center gap-4">
      {onOpenNav && (
        <button
          type="button"
          onClick={onOpenNav}
          className="lg:hidden p-2 rounded-lg text-muted-foreground hover:text-foreground hover:bg-border/50"
          aria-label="Abrir menu"
        >
          <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
          </svg>
        </button>
      )}
      {title && <h1 className="text-xl font-semibold">{title}</h1>}
    </header>
  );
}
