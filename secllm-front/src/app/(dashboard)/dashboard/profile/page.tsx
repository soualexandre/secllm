"use client";

import { useRouter } from "next/navigation";
import { Card, Button } from "@/components/ui";
import { useAuth } from "@/hooks/useAuth";

export default function ProfilePage() {
  const router = useRouter();
  const { user, logout } = useAuth();

  async function handleLogout() {
    await logout();
    router.push("/login");
    router.refresh();
  }

  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-semibold">Perfil</h2>
      <Card title="Dados do usuário">
        <div className="space-y-3 text-sm">
          {user?.email ? (
            <>
              <div className="grid grid-cols-[120px_1fr] gap-2">
                <span className="text-muted-foreground">Email</span>
                <span className="font-medium">{user.email}</span>
              </div>
              {user.name != null && user.name !== "" && (
                <div className="grid grid-cols-[120px_1fr] gap-2">
                  <span className="text-muted-foreground">Nome</span>
                  <span className="font-medium">{user.name}</span>
                </div>
              )}
              {user.role && (
                <div className="grid grid-cols-[120px_1fr] gap-2">
                  <span className="text-muted-foreground">Função</span>
                  <span className="font-medium capitalize">{user.role}</span>
                </div>
              )}
            </>
          ) : (
            <p className="text-muted-foreground">Carregando dados…</p>
          )}
        </div>
        <div className="mt-6 pt-4 border-t border-border">
          <Button variant="danger" onClick={handleLogout}>
            Sair da aplicação
          </Button>
        </div>
      </Card>
    </div>
  );
}
