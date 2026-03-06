"use client";

import { useState } from "react";
import { Sidebar, NavContent, Header, Breadcrumb, CommandPalette } from "@/components/layout";
import { Drawer } from "@/components/ui";

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const [mobileNavOpen, setMobileNavOpen] = useState(false);

  return (
    <div className="flex min-h-screen">
      <Sidebar />
      <main className="flex-1 flex flex-col min-w-0">
        <Header onOpenNav={() => setMobileNavOpen(true)} />
        <div className="flex-1 p-4 sm:p-6">
          <Breadcrumb />
          {children}
        </div>
      </main>
      <Drawer
        open={mobileNavOpen}
        onClose={() => setMobileNavOpen(false)}
        title="Menu"
        side="left"
      >
        <div className="flex flex-col min-h-[60vh]">
          <NavContent onNavigate={() => setMobileNavOpen(false)} />
        </div>
      </Drawer>
      <CommandPalette />
    </div>
  );
}
