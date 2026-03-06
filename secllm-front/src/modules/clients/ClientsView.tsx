"use client";

import { useState } from "react";
import axios from "axios";
import {
  Button,
  Card,
  Modal,
  Input,
  Select,
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  TableCell,
  Badge,
  Drawer,
} from "@/components/ui";
import {
  useClientsList,
  useProvidersList,
  useCreateClient,
  usePutApiKey,
  useDeleteApiKey,
  usePutClientSecret,
  useDeleteClientSecret,
} from "@/hooks/useClients";
import { createClientSchema } from "@/lib/validators";
import type { CreateClientInput } from "@/lib/validators";
import type { ListClientItem } from "@/types";
import { getCredentials } from "@/services/clients.service";

const FALLBACK_PROVIDERS = ["openai", "anthropic", "gemini"] as const;

type ConfirmRemove = { type: "key"; provider: string } | { type: "secret" };

function EyeIcon({ className }: { className?: string }) {
  return (
    <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden>
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
    </svg>
  );
}

export function ClientsView() {
  const [open, setOpen] = useState(false);
  const [name, setName] = useState("");
  const [createdCredentials, setCreatedCredentials] = useState<{
    client_id: string;
    client_secret: string;
    name?: string;
  } | null>(null);
  const [validationErrors, setValidationErrors] = useState<Record<string, string[]>>({});
  const [manageClient, setManageClient] = useState<ListClientItem | null>(null);

  const clientsList = useClientsList();
  const providersList = useProvidersList();
  const providers = providersList.data ?? [...FALLBACK_PROVIDERS];
  const clients = clientsList.data ?? [];
  const [manageKeys, setManageKeys] = useState<Record<string, string>>({});
  const [selectedProvider, setSelectedProvider] = useState<string>("");
  const [manageSecret, setManageSecret] = useState("");
  const [manageError, setManageError] = useState("");

  const [confirmRemove, setConfirmRemove] = useState<ConfirmRemove | null>(null);
  const [revealedCredentials, setRevealedCredentials] = useState<Record<string, string | null>>({});
  const [loadingReveal, setLoadingReveal] = useState<string | null>(null);

  const createClientMutation = useCreateClient();
  const putKeyMutation = usePutApiKey();
  const deleteKeyMutation = useDeleteApiKey();
  const putSecretMutation = usePutClientSecret();
  const deleteSecretMutation = useDeleteClientSecret();

  function getApiErrorMessage(err: unknown): string {
    if (axios.isAxiosError(err)) {
      const d = err.response?.data;
      if (d && typeof d === "object") {
        const o = d as { error?: string; detail?: string; message?: string };
        const msg = o.error ?? o.detail ?? o.message;
        if (typeof msg === "string") return msg;
      }
      return err.message || "Request failed";
    }
    return err instanceof Error ? err.message : "Failed to create client";
  }

  async function handleCreate(e: React.FormEvent) {
    e.preventDefault();
    setValidationErrors({});
    createClientMutation.reset();
    const parsed = createClientSchema.safeParse({ name: name.trim() || undefined });
    if (!parsed.success) {
      const fieldErrors: Record<string, string[]> = {};
      for (const [k, v] of Object.entries(parsed.error.flatten().fieldErrors)) {
        fieldErrors[k] = (v as string[]).filter(Boolean);
      }
      setValidationErrors(fieldErrors);
      return;
    }
    const data = parsed.data as CreateClientInput;
    try {
      const res = await createClientMutation.mutateAsync({ name: data.name });
      setCreatedCredentials({
        client_id: res.client_id,
        client_secret: res.client_secret,
        name: res.name,
      });
      await clientsList.refetch();
      setOpen(false);
      setName("");
    } catch (err) {
      setValidationErrors({});
    }
  }

  async function handlePutKey(provider: string, apiKey: string) {
    if (!manageClient || !apiKey.trim()) return;
    setManageError("");
    try {
      await putKeyMutation.mutateAsync({
        clientId: manageClient.client_id,
        provider,
        apiKey: apiKey.trim(),
      });
      const { data: nextList } = await clientsList.refetch();
      const updated = nextList?.find((c) => c.client_id === manageClient.client_id);
      if (updated) setManageClient(updated);
      setManageKeys((prev) => {
        const next = { ...prev };
        delete next[provider];
        return next;
      });
    } catch (err) {
      setManageError(getApiErrorMessage(err));
    }
  }

  async function handleDeleteKey(provider: string) {
    if (!manageClient) return;
    setManageError("");
    try {
      await deleteKeyMutation.mutateAsync({
        clientId: manageClient.client_id,
        provider,
      });
      const { data: nextList } = await clientsList.refetch();
      const updated = nextList?.find((c) => c.client_id === manageClient.client_id);
      if (updated) setManageClient(updated);
    } catch (err) {
      setManageError(getApiErrorMessage(err));
    }
  }

  async function handlePutSecret() {
    if (!manageClient || !manageSecret.trim()) return;
    setManageError("");
    try {
      await putSecretMutation.mutateAsync({
        clientId: manageClient.client_id,
        clientSecret: manageSecret.trim(),
      });
      const { data: nextList } = await clientsList.refetch();
      const updated = nextList?.find((c) => c.client_id === manageClient.client_id);
      if (updated) setManageClient(updated);
      setManageSecret("");
    } catch (err) {
      setManageError(getApiErrorMessage(err));
    }
  }

  async function handleDeleteSecret() {
    if (!manageClient) return;
    setManageError("");
    try {
      await deleteSecretMutation.mutateAsync(manageClient.client_id);
      const { data: nextList } = await clientsList.refetch();
      const updated = nextList?.find((c) => c.client_id === manageClient.client_id);
      if (updated) setManageClient(updated);
    } catch (err) {
      setManageError(getApiErrorMessage(err));
    }
  }

  function openConfirmRemoveKey(provider: string) {
    setConfirmRemove({ type: "key", provider });
  }

  function openConfirmRemoveSecret() {
    setConfirmRemove({ type: "secret" });
  }

  async function handleConfirmRemove() {
    if (!confirmRemove || !manageClient) return;
    try {
      if (confirmRemove.type === "key") {
        await handleDeleteKey(confirmRemove.provider);
      } else {
        await handleDeleteSecret();
      }
      setConfirmRemove(null);
    } catch {
      // erro já exibido em manageError pelos handlers
    }
  }

  async function handleRevealKey(provider: string) {
    if (!manageClient) return;
    setLoadingReveal(provider);
    setManageError("");
    try {
      const cred = await getCredentials(manageClient.client_id);
      const value = cred.keys?.[provider] ?? null;
      setRevealedCredentials((prev) => ({ ...prev, [provider]: value }));
    } catch (err) {
      setManageError(getApiErrorMessage(err));
    } finally {
      setLoadingReveal(null);
    }
  }

  async function handleRevealSecret() {
    if (!manageClient) return;
    setLoadingReveal("secret");
    setManageError("");
    try {
      const cred = await getCredentials(manageClient.client_id);
      setRevealedCredentials((prev) => ({ ...prev, secret: cred.client_secret ?? null }));
    } catch (err) {
      setManageError(getApiErrorMessage(err));
    } finally {
      setLoadingReveal(null);
    }
  }

  function hideRevealed(which: string) {
    setRevealedCredentials((prev) => {
      const next = { ...prev };
      delete next[which];
      return next;
    });
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h2 className="text-2xl font-semibold">Clients</h2>
        <Button onClick={() => setOpen(true)}>Create client</Button>
      </div>
      <Card>
        {clientsList.isLoading ? (
          <p className="text-muted-foreground text-sm">Loading clients…</p>
        ) : clientsList.isError ? (
          <p className="text-sm text-danger">
            {getApiErrorMessage(clientsList.error)}
          </p>
        ) : clients.length === 0 ? (
          <p className="text-muted-foreground text-sm">
            No clients yet. Create one to manage API keys and secrets.
          </p>
        ) : (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Client ID</TableHead>
                <TableHead>Name</TableHead>
                <TableHead>Keys</TableHead>
                <TableHead>Secret</TableHead>
                <TableHead>Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {clients.map((c) => (
                <TableRow key={c.client_id}>
                  <TableCell className="font-mono text-sm">{c.client_id}</TableCell>
                  <TableCell>{c.name ?? "—"}</TableCell>
                  <TableCell>
                    <div className="flex gap-1">
                      {c.keys.map((k) => (
                        <Badge key={k} variant="primary">
                          {k}
                        </Badge>
                      ))}
                      {c.keys.length === 0 && "—"}
                    </div>
                  </TableCell>
                  <TableCell>
                    {c.has_secret ? <Badge variant="success">Set</Badge> : "—"}
                  </TableCell>
                  <TableCell>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => {
                        setManageClient(c);
                        setManageError("");
                        setManageKeys({});
                        setSelectedProvider(providers[0] ?? "");
                        setManageSecret("");
                        setConfirmRemove(null);
                        setRevealedCredentials({});
                      }}
                    >
                      Manage
                    </Button>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        )}
      </Card>

      <Modal
        open={open}
        onClose={() => setOpen(false)}
        title="Create client"
        footer={
          <>
            <Button variant="secondary" onClick={() => setOpen(false)}>
              Cancel
            </Button>
            <Button
              onClick={(e) => handleCreate(e as unknown as React.FormEvent)}
              disabled={createClientMutation.isPending}
            >
              {createClientMutation.isPending ? "Creating…" : "Create"}
            </Button>
          </>
        }
      >
        <form onSubmit={handleCreate} className="space-y-4">
          <Input
            label="Name (optional)"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="My app"
          />
          {validationErrors.name?.length ? (
            <p className="text-sm text-danger">{validationErrors.name[0]}</p>
          ) : null}
          {createClientMutation.isError && (
            <p className="text-sm text-danger">
              {getApiErrorMessage(createClientMutation.error)}
            </p>
          )}
        </form>
      </Modal>

      <Modal
        open={!!createdCredentials}
        onClose={() => setCreatedCredentials(null)}
        title="Cliente criado"
        footer={
          <Button onClick={() => setCreatedCredentials(null)}>
            Guardei as credenciais
          </Button>
        }
      >
        {createdCredentials && (
          <div className="space-y-4">
            <p className="text-sm text-muted-foreground">
              Guarde o client secret; ele não será exibido novamente nesta tela.
            </p>
            <div>
              <label className="text-sm font-medium block mb-1">Client ID</label>
              <div className="flex gap-2">
                <input
                  readOnly
                  type="text"
                  value={createdCredentials.client_id}
                  className="flex-1 px-2 py-1.5 text-sm rounded bg-muted border border-border font-mono"
                />
                <Button
                  type="button"
                  size="sm"
                  variant="secondary"
                  onClick={() => navigator.clipboard.writeText(createdCredentials.client_id)}
                >
                  Copiar
                </Button>
              </div>
            </div>
            <div>
              <label className="text-sm font-medium block mb-1">Client secret</label>
              <div className="flex gap-2">
                <input
                  readOnly
                  type="text"
                  value={createdCredentials.client_secret}
                  className="flex-1 px-2 py-1.5 text-sm rounded bg-muted border border-border font-mono"
                />
                <Button
                  type="button"
                  size="sm"
                  variant="secondary"
                  onClick={() => navigator.clipboard.writeText(createdCredentials.client_secret)}
                >
                  Copiar
                </Button>
              </div>
            </div>
          </div>
        )}
      </Modal>

      <Drawer
        open={!!manageClient}
        onClose={() => setManageClient(null)}
        title={manageClient ? `Manage: ${manageClient.client_id}` : "Manage"}
      >
        {manageClient && (
          <div className="space-y-6">
            <p className="text-sm text-muted-foreground">
              {manageClient.name ? `Name: ${manageClient.name}` : "No name"}
            </p>

            <div>
              <h3 className="text-sm font-medium mb-2">API Keys</h3>
              <Select
                label="Provedor"
                value={selectedProvider}
                onChange={(e) => setSelectedProvider(e.target.value)}
                options={[
                  { value: "", label: "Selecione um provedor" },
                  ...providers.map((p) => ({ value: p, label: p.charAt(0).toUpperCase() + p.slice(1) })),
                ]}
              />
              {selectedProvider && (
                <div className="mt-3 space-y-2">
                  {manageClient.keys.includes(selectedProvider) ? (
                    <div className="space-y-1">
                      <div className="flex items-center gap-2 flex-wrap">
                        <Badge variant="primary">Configurado</Badge>
                        <button
                          type="button"
                          onClick={() =>
                            revealedCredentials[selectedProvider]
                              ? hideRevealed(selectedProvider)
                              : handleRevealKey(selectedProvider)
                          }
                          disabled={loadingReveal === selectedProvider}
                          className="text-muted-foreground hover:text-foreground p-1"
                          aria-label={revealedCredentials[selectedProvider] ? "Ocultar" : "Ver"}
                        >
                          <EyeIcon className="w-5 h-5" />
                        </button>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => openConfirmRemoveKey(selectedProvider)}
                          disabled={deleteKeyMutation.isPending}
                        >
                          Remover chave
                        </Button>
                      </div>
                      {revealedCredentials[selectedProvider] != null && (
                        <div className="flex gap-2">
                          <input
                            readOnly
                            type="text"
                            value={revealedCredentials[selectedProvider] ?? ""}
                            className="flex-1 px-2 py-1 text-sm rounded bg-muted border border-border font-mono"
                          />
                          <Button variant="ghost" size="sm" onClick={() => hideRevealed(selectedProvider)}>
                            Ocultar
                          </Button>
                        </div>
                      )}
                      {loadingReveal === selectedProvider && (
                        <span className="text-xs text-muted-foreground">Carregando…</span>
                      )}
                    </div>
                  ) : (
                    <div className="flex gap-2">
                      <Input
                        type="password"
                        revealable
                        placeholder="Cole a API key do provedor"
                        value={manageKeys[selectedProvider] ?? ""}
                        onChange={(e) =>
                          setManageKeys((prev) => ({ ...prev, [selectedProvider]: e.target.value }))
                        }
                        className="flex-1"
                      />
                      <Button
                        size="sm"
                        onClick={() => handlePutKey(selectedProvider, manageKeys[selectedProvider] ?? "")}
                        disabled={
                          putKeyMutation.isPending || !(manageKeys[selectedProvider] ?? "").trim()
                        }
                      >
                        Adicionar
                      </Button>
                    </div>
                  )}
                </div>
              )}
              <div className="mt-4">
                <h4 className="text-xs font-medium text-muted-foreground mb-2">Chaves já adicionadas</h4>
                {manageClient.keys.length === 0 ? (
                  <p className="text-sm text-muted-foreground">Nenhuma chave configurada. Selecione um provedor e adicione uma API key acima.</p>
                ) : (
                  <ul className="space-y-2">
                    {manageClient.keys.map((key) => (
                      <li key={key} className="flex items-center gap-2 flex-wrap">
                        <Badge variant="primary">{key}</Badge>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => openConfirmRemoveKey(key)}
                          disabled={deleteKeyMutation.isPending}
                        >
                          Remover
                        </Button>
                      </li>
                    ))}
                  </ul>
                )}
              </div>
            </div>

            <div>
              <h3 className="text-sm font-medium mb-2">Client secret</h3>
              {manageClient.has_secret ? (
                <div className="space-y-1">
                  <div className="flex items-center gap-2">
                    <button
                      type="button"
                      onClick={() =>
                        revealedCredentials.secret != null
                          ? hideRevealed("secret")
                          : handleRevealSecret()
                      }
                      disabled={loadingReveal === "secret"}
                      className="text-muted-foreground hover:text-foreground p-1"
                      aria-label={revealedCredentials.secret != null ? "Ocultar" : "Ver"}
                    >
                      <EyeIcon className="w-5 h-5" />
                    </button>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={openConfirmRemoveSecret}
                      disabled={deleteSecretMutation.isPending}
                    >
                      Remove secret
                    </Button>
                  </div>
                  {revealedCredentials.secret != null && (
                    <div className="flex items-center gap-2">
                      <input
                        readOnly
                        type="text"
                        value={revealedCredentials.secret}
                        className="flex-1 px-2 py-1 text-sm rounded bg-muted border border-border font-mono"
                      />
                      <Button variant="ghost" size="sm" onClick={() => hideRevealed("secret")}>
                        Ocultar
                      </Button>
                    </div>
                  )}
                  {loadingReveal === "secret" && (
                    <span className="text-xs text-muted-foreground">Carregando…</span>
                  )}
                </div>
              ) : (
                <div className="flex gap-2">
                  <Input
                    type="password"
                    revealable
                    placeholder="Secret"
                    value={manageSecret}
                    onChange={(e) => setManageSecret(e.target.value)}
                    className="flex-1"
                  />
                  <Button
                    size="sm"
                    onClick={handlePutSecret}
                    disabled={putSecretMutation.isPending || !manageSecret.trim()}
                  >
                    Set secret
                  </Button>
                </div>
              )}
            </div>

            {manageError && (
              <p className="text-sm text-danger">{manageError}</p>
            )}
          </div>
        )}
      </Drawer>

      <Modal
        open={!!confirmRemove}
        onClose={() => setConfirmRemove(null)}
        title="Confirmar remoção"
        footer={
          <>
            <Button variant="secondary" onClick={() => setConfirmRemove(null)}>
              Cancelar
            </Button>
            <Button
              variant="danger"
              onClick={handleConfirmRemove}
              disabled={
                deleteKeyMutation.isPending ||
                deleteSecretMutation.isPending
              }
            >
              {deleteKeyMutation.isPending || deleteSecretMutation.isPending
                ? "Removendo…"
                : "Sim, remover"}
            </Button>
          </>
        }
      >
        <p className="text-foreground">
          Tem certeza que deseja remover
          {confirmRemove?.type === "key" ? (
            <> a chave <strong>{confirmRemove.provider}</strong>?</>
          ) : (
            <> o client secret?</>
          )}
        </p>
      </Modal>
    </div>
  );
}
