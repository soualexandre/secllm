"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { Command } from "cmdk";
import { useUIStore } from "@/stores/ui.store";

export function CommandPalette() {
  const router = useRouter();
  const { commandPaletteOpen, setCommandPaletteOpen } = useUIStore();

  useEffect(() => {
    const down = (e: KeyboardEvent) => {
      if (e.key === "k" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        setCommandPaletteOpen(!commandPaletteOpen);
      }
    };
    document.addEventListener("keydown", down);
    return () => document.removeEventListener("keydown", down);
  }, [commandPaletteOpen, setCommandPaletteOpen]);

  if (!commandPaletteOpen) return null;

  return (
    <div className="fixed inset-0 z-50 bg-black/70 flex items-start justify-center pt-[20vh]">
      <div
        className="absolute inset-0"
        onClick={() => setCommandPaletteOpen(false)}
        aria-hidden
      />
      <Command
        className="relative w-full max-w-md rounded-xl border border-border bg-card shadow-xl overflow-hidden"
        onValueChange={(value) => {
          setCommandPaletteOpen(false);
          if (value) router.push(value);
        }}
      >
        <Command.Input
          placeholder="Search or run a command…"
          className="w-full px-4 py-3 bg-transparent border-b border-border text-foreground placeholder:text-muted-foreground focus:outline-none"
        />
        <Command.List className="max-h-72 overflow-auto p-2">
          <Command.Group heading="Navigation">
            <Command.Item value="/dashboard" className="px-4 py-2 rounded-lg cursor-pointer hover:bg-border/50">
              Dashboard
            </Command.Item>
            <Command.Item value="/dashboard/clients" className="px-4 py-2 rounded-lg cursor-pointer hover:bg-border/50">
              Clients
            </Command.Item>
            <Command.Item value="/dashboard/governance" className="px-4 py-2 rounded-lg cursor-pointer hover:bg-border/50">
              Governance
            </Command.Item>
            <Command.Item value="/dashboard/billing" className="px-4 py-2 rounded-lg cursor-pointer hover:bg-border/50">
              Billing
            </Command.Item>
            <Command.Item value="/dashboard/settings" className="px-4 py-2 rounded-lg cursor-pointer hover:bg-border/50">
              Settings
            </Command.Item>
          </Command.Group>
        </Command.List>
      </Command>
    </div>
  );
}
