"use client";

import { useRouter } from "next/navigation";
import { Card, Button, Select } from "@/components/ui";
import { useAuth } from "@/hooks/useAuth";
import { useTheme } from "@/stores/theme.store";
import type { Theme } from "@/stores/theme.store";

const THEME_OPTIONS: { value: Theme; label: string }[] = [
  { value: "system", label: "Sistema (padrão)" },
  { value: "light", label: "Claro" },
  { value: "dark", label: "Escuro" },
];

export default function SettingsPage() {
  const router = useRouter();
  const { logout } = useAuth();
  const { theme, setTheme } = useTheme();

  async function handleLogout() {
    await logout();
    router.push("/login");
    router.refresh();
  }

  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-semibold">Settings</h2>

      <Card title="Aparência">
        <Select
          label="Modo"
          value={theme}
          onChange={(e) => setTheme(e.target.value as Theme)}
          options={THEME_OPTIONS}
        />
        <p className="text-xs text-muted-foreground mt-2">
          &quot;Sistema&quot; usa a preferência do seu sistema operacional (claro ou escuro).
        </p>
      </Card>

      <Card title="Account">
        <p className="text-sm text-muted-foreground mb-4">API: {process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:3010"}</p>
        <Button variant="danger" onClick={handleLogout}>
          Sign out
        </Button>
      </Card>
    </div>
  );
}
