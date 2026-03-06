"use client";

import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import { useAuth } from "@/hooks/useAuth";
import { Button } from "@/components/ui";

const nav = [
  { href: "/dashboard", label: "Dashboard" },
  { href: "/dashboard/clients", label: "Clients" },
  { href: "/dashboard/governance", label: "Governance" },
  { href: "/dashboard/billing", label: "Billing" },
  { href: "/dashboard/profile", label: "Perfil" },
  { href: "/dashboard/settings", label: "Settings" },
];

export function Sidebar() {
  const pathname = usePathname();
  const router = useRouter();
  const { user, logout } = useAuth();

  async function handleLogout() {
    await logout();
    router.push("/login");
    router.refresh();
  }

  return (
    <aside className="w-56 border-r border-border bg-card flex flex-col min-h-screen">
      <div className="p-4 border-b border-border">
        <Link href="/dashboard" className="text-lg font-semibold text-foreground">
          SecLLM
        </Link>
      </div>
      <nav className="flex-1 p-4 space-y-1">
        {nav.map((item) => {
          const isActive = pathname === item.href || (item.href !== "/dashboard" && pathname.startsWith(item.href));
          return (
            <Link
              key={item.href}
              href={item.href}
              className={`block px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                isActive ? "bg-primary text-white" : "text-muted-foreground hover:text-foreground hover:bg-border/50"
              }`}
            >
              {item.label}
            </Link>
          );
        })}
      </nav>
      <div className="p-4 border-t border-border space-y-2">
        {user?.email && (
          <div className="text-xs text-muted-foreground truncate" title={user.email}>
            {user.email}
          </div>
        )}
        <Button variant="ghost" size="sm" className="w-full justify-start" onClick={handleLogout}>
          Sair
        </Button>
      </div>
    </aside>
  );
}
