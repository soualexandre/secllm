"use client";

import { useCallback, useEffect, useState } from "react";
import {
  Card,
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  TableCell,
  Badge,
  Button,
  Drawer,
  Input,
  Select,
} from "@/components/ui";
import { getLogs, type LogsParams } from "@/services/logs.service";
import type { LogEntry } from "@/types/logs";

const PAGE_SIZE = 50;

const SORT_OPTIONS = [
  { value: "created_at", label: "Data" },
  { value: "status", label: "Status" },
  { value: "latency_ms", label: "Latência" },
  { value: "prompt_tokens", label: "Tokens" },
  { value: "client_id", label: "Client" },
  { value: "provider", label: "Provider" },
];

const STATUS_OPTIONS = [
  { value: "", label: "Todos" },
  { value: "ok", label: "OK" },
  { value: "error", label: "Erro" },
];

const PROVIDER_OPTIONS = [
  { value: "", label: "Todos" },
  { value: "OpenAI", label: "OpenAI" },
  { value: "Anthropic", label: "Anthropic" },
  { value: "Gemini", label: "Gemini" },
];

function useDebounce<T>(value: T, delay: number): T {
  const [debouncedValue, setDebouncedValue] = useState(value);
  useEffect(() => {
    const t = setTimeout(() => setDebouncedValue(value), delay);
    return () => clearTimeout(t);
  }, [value, delay]);
  return debouncedValue;
}

function tryFormatJson(text: string): string {
  try {
    const parsed = JSON.parse(text);
    return JSON.stringify(parsed, null, 2);
  } catch {
    return text;
  }
}

export default function LogsPage() {
  const [items, setItems] = useState<LogEntry[]>([]);
  const [total, setTotal] = useState(0);
  const [offset, setOffset] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [detail, setDetail] = useState<LogEntry | null>(null);

  const [clientIdInput, setClientIdInput] = useState("");
  const [provider, setProvider] = useState("");
  const [status, setStatus] = useState("");
  const [sort, setSort] = useState("created_at");
  const [order, setOrder] = useState<"asc" | "desc">("desc");
  const [copiedId, setCopiedId] = useState<string | null>(null);

  const clientIdDebounced = useDebounce(clientIdInput.trim(), 400);

  const copyClientId = useCallback((clientId: string) => {
    navigator.clipboard.writeText(clientId).then(() => {
      setCopiedId(clientId);
      setTimeout(() => setCopiedId(null), 2000);
    });
  }, []);

  const searchByClientId = useCallback((clientId: string) => {
    setClientIdInput(clientId);
    setDetail(null);
  }, []);

  const hasActiveFilters = clientIdDebounced !== "" || provider !== "" || status !== "";

  const fetchLogs = useCallback(
    (overrides?: Partial<LogsParams>) => {
      setLoading(true);
      const params: LogsParams = {
        limit: PAGE_SIZE,
        offset: overrides?.offset ?? offset,
        client_id: clientIdDebounced || undefined,
        provider: provider || undefined,
        status: status || undefined,
        sort,
        order,
        ...overrides,
      };
      getLogs(params)
        .then((res) => {
          setItems(res.items);
          setTotal(res.total);
          if (overrides?.offset !== undefined) setOffset(overrides.offset);
        })
        .catch((e) => setError(e instanceof Error ? e.message : "Erro ao carregar logs"))
        .finally(() => setLoading(false));
    },
    [clientIdDebounced, provider, status, sort, order, offset]
  );

  // Quando o termo de busca (debounced) muda, voltar para a primeira página
  useEffect(() => {
    setOffset(0);
  }, [clientIdDebounced]);

  useEffect(() => {
    fetchLogs();
  }, [clientIdDebounced, provider, status, sort, order, offset]);

  const goToPage = (newOffset: number) => {
    setOffset(newOffset);
    fetchLogs({ offset: newOffset });
  };

  const clearFilters = () => {
    setClientIdInput("");
    setProvider("");
    setStatus("");
    setSort("created_at");
    setOrder("desc");
    setOffset(0);
  };

  const totalPages = Math.ceil(total / PAGE_SIZE) || 1;
  const currentPage = Math.floor(offset / PAGE_SIZE) + 1;

  if (error) {
    return (
      <div className="space-y-6">
        <h2 className="text-2xl font-semibold">Logs de auditoria</h2>
        <Card>
          <p className="text-danger">{error}</p>
        </Card>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <h2 className="text-2xl font-semibold">Logs de auditoria</h2>
      </div>

      <Card>
        {/* Barra de filtros e ordenação */}
        <div className="space-y-4 mb-6">
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-3">
            <div className="lg:col-span-2">
              <label className="block text-xs font-medium text-muted-foreground mb-1">
                Buscar por client_id
              </label>
              <Input
                type="text"
                placeholder="Ex: cli_abc123..."
                value={clientIdInput}
                onChange={(e) => setClientIdInput(e.target.value)}
                className="w-full"
              />
            </div>
            <div>
              <label className="block text-xs font-medium text-muted-foreground mb-1">Provider</label>
              <Select
                options={PROVIDER_OPTIONS}
                value={provider}
                onChange={(e) => {
                  setProvider(e.target.value);
                  setOffset(0);
                }}
                className="w-full"
              />
            </div>
            <div>
              <label className="block text-xs font-medium text-muted-foreground mb-1">Status</label>
              <Select
                options={STATUS_OPTIONS}
                value={status}
                onChange={(e) => {
                  setStatus(e.target.value);
                  setOffset(0);
                }}
                className="w-full"
              />
            </div>
            <div className="sm:col-span-2 lg:col-span-1 flex flex-col sm:flex-row gap-2">
              <div className="flex-1">
                <label className="block text-xs font-medium text-muted-foreground mb-1">
                  Ordenar por
                </label>
                <Select
                  options={SORT_OPTIONS}
                  value={sort}
                  onChange={(e) => {
                    setSort(e.target.value);
                    setOffset(0);
                  }}
                  className="w-full"
                />
              </div>
              <div className="flex items-end">
                <button
                  type="button"
                  onClick={() => {
                    setOrder(order === "desc" ? "asc" : "desc");
                    setOffset(0);
                  }}
                  className="px-3 py-2 rounded-lg border border-border bg-card text-sm font-medium text-foreground hover:bg-border/50 transition-colors"
                  title={order === "desc" ? "Mais recentes primeiro (clique para inverter)" : "Mais antigos primeiro"}
                >
                  {order === "desc" ? "↓" : "↑"}
                </button>
              </div>
            </div>
          </div>

          {/* Chips de filtros ativos + Limpar */}
          <div className="flex flex-wrap items-center gap-2">
            {hasActiveFilters && (
              <Button variant="ghost" size="sm" onClick={clearFilters}>
                Limpar filtros
              </Button>
            )}
            {clientIdDebounced && (
              <span className="inline-flex items-center gap-1 px-2.5 py-1 rounded-md bg-primary/15 text-primary text-xs font-medium">
                client_id: {clientIdDebounced}
                <button
                  type="button"
                  onClick={() => setClientIdInput("")}
                  className="hover:bg-primary/20 rounded p-0.5"
                  aria-label="Remover"
                >
                  ×
                </button>
              </span>
            )}
            {provider && (
              <span className="inline-flex items-center gap-1 px-2.5 py-1 rounded-md bg-primary/15 text-primary text-xs font-medium">
                provider: {provider}
                <button
                  type="button"
                  onClick={() => setProvider("")}
                  className="hover:bg-primary/20 rounded p-0.5"
                  aria-label="Remover"
                >
                  ×
                </button>
              </span>
            )}
            {status && (
              <span className="inline-flex items-center gap-1 px-2.5 py-1 rounded-md bg-primary/15 text-primary text-xs font-medium">
                status: {status}
                <button
                  type="button"
                  onClick={() => setStatus("")}
                  className="hover:bg-primary/20 rounded p-0.5"
                  aria-label="Remover"
                >
                  ×
                </button>
              </span>
            )}
          </div>
        </div>

        <p className="text-muted-foreground text-sm mb-4">
          {total.toLocaleString()} resultado{total !== 1 ? "s" : ""}. Clique em &quot;Ver&quot; para detalhes.
        </p>

        {loading ? (
          <div className="space-y-2">
            {[1, 2, 3, 4, 5].map((i) => (
              <div key={i} className="h-10 bg-muted rounded animate-pulse" />
            ))}
          </div>
        ) : items.length === 0 ? (
          <p className="text-muted-foreground text-sm py-8 text-center">
            Nenhum log encontrado. Ajuste os filtros ou aguarde novos registros.
          </p>
        ) : (
          <>
            <div className="overflow-x-auto -mx-2 sm:mx-0">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead className="whitespace-nowrap">Data</TableHead>
                    <TableHead className="hidden sm:table-cell">client_id</TableHead>
                    <TableHead className="hidden md:table-cell">provider</TableHead>
                    <TableHead className="hidden lg:table-cell">model</TableHead>
                    <TableHead>status</TableHead>
                    <TableHead className="hidden lg:table-cell text-right">tokens</TableHead>
                    <TableHead className="hidden xl:table-cell text-right">latência</TableHead>
                    <TableHead className="w-10 sm:w-20" aria-label="Ver detalhes">
                      <span className="sr-only">Ver detalhes</span>
                    </TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {items.map((row) => (
                    <TableRow key={row.request_id}>
                      <TableCell className="whitespace-nowrap text-muted-foreground">
                        {row.created_at_formatted}
                      </TableCell>
                      <TableCell className="hidden sm:table-cell">
                        <div className="flex items-center gap-1.5 max-w-[140px]">
                          <span className="font-mono text-xs truncate" title={row.client_id}>
                            {row.client_id}
                          </span>
                          <button
                            type="button"
                            onClick={(e) => {
                              e.stopPropagation();
                              copyClientId(row.client_id);
                            }}
                            className="shrink-0 p-1 rounded hover:bg-muted text-muted-foreground hover:text-foreground transition-colors"
                            title="Copiar client_id para buscar"
                            aria-label="Copiar client_id"
                          >
                            {copiedId === row.client_id ? (
                              <span className="text-xs text-success">✓</span>
                            ) : (
                              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
                            )}
                          </button>
                        </div>
                      </TableCell>
                      <TableCell className="hidden md:table-cell">{row.provider}</TableCell>
                      <TableCell className="hidden lg:table-cell text-muted-foreground">
                        {row.model ?? "—"}
                      </TableCell>
                      <TableCell>
                        <Badge variant={row.status === "ok" ? "success" : "danger"}>
                          {row.status}
                        </Badge>
                      </TableCell>
                      <TableCell className="hidden lg:table-cell text-right text-muted-foreground">
                        {row.prompt_tokens != null && row.completion_tokens != null
                          ? `${row.prompt_tokens} / ${row.completion_tokens}`
                          : "—"}
                      </TableCell>
                      <TableCell className="hidden xl:table-cell text-right">
                        {row.latency_ms != null ? `${row.latency_ms} ms` : "—"}
                      </TableCell>
                      <TableCell className="text-center">
                        <Button
                          variant="ghost"
                          size="sm"
                          aria-label="Ver detalhes"
                          onClick={() => setDetail(row)}
                        >
                          Ver
                        </Button>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>
            {totalPages > 1 && (
              <div className="flex flex-wrap items-center justify-between gap-2 mt-4 pt-4 border-t border-border">
                <p className="text-sm text-muted-foreground">
                  Página {currentPage} de {totalPages}
                </p>
                <div className="flex gap-2">
                  <Button
                    variant="secondary"
                    size="sm"
                    disabled={offset === 0}
                    onClick={() => goToPage(Math.max(0, offset - PAGE_SIZE))}
                  >
                    Anterior
                  </Button>
                  <Button
                    variant="secondary"
                    size="sm"
                    disabled={offset + PAGE_SIZE >= total}
                    onClick={() => goToPage(offset + PAGE_SIZE)}
                  >
                    Próxima
                  </Button>
                </div>
              </div>
            )}
          </>
        )}
      </Card>

      <Drawer
        open={detail !== null}
        onClose={() => setDetail(null)}
        title="Detalhes do log"
        side="right"
      >
        {detail && (
          <div className="space-y-6 text-sm">
            {detail.request_body != null && detail.request_body.length > 0 && (
              <div>
                <h4 className="font-semibold text-foreground mb-2">Prompt de entrada</h4>
                <pre className="p-4 rounded-lg bg-muted/50 border border-border overflow-x-auto text-xs whitespace-pre-wrap break-words max-h-48 overflow-y-auto">
                  {tryFormatJson(detail.request_body)}
                </pre>
              </div>
            )}

            {detail.response_body != null && detail.response_body.length > 0 && (
              <div>
                <h4 className="font-semibold text-foreground mb-2">Resposta (saída da LLM)</h4>
                <pre className="p-4 rounded-lg bg-muted/50 border border-border overflow-x-auto text-xs whitespace-pre-wrap break-words max-h-48 overflow-y-auto">
                  {tryFormatJson(detail.response_body)}
                </pre>
              </div>
            )}

            {((detail.request_body == null || detail.request_body.length === 0) &&
              (detail.response_body == null || detail.response_body.length === 0)) && (
              <p className="text-muted-foreground">
                Entrada/saída não armazenadas (rode a migração 03_add_request_response_body.sql no ClickHouse).
              </p>
            )}

            <hr className="border-border" />
            <dl className="grid grid-cols-1 sm:grid-cols-2 gap-2">
              <dt className="text-muted-foreground">Data</dt>
              <dd>{detail.created_at_formatted}</dd>
              <dt className="text-muted-foreground">request_id</dt>
              <dd className="font-mono text-xs break-all">{detail.request_id}</dd>
              <dt className="text-muted-foreground">client_id</dt>
              <dd className="font-mono flex items-center gap-2 flex-wrap">
                <span className="break-all">{detail.client_id}</span>
                <span className="flex items-center gap-1">
                  <button
                    type="button"
                    onClick={() => copyClientId(detail.client_id)}
                    className="px-2 py-1 rounded border border-border bg-card text-xs text-muted-foreground hover:text-foreground hover:bg-muted transition-colors"
                  >
                    {copiedId === detail.client_id ? "Copiado!" : "Copiar"}
                  </button>
                  <button
                    type="button"
                    onClick={() => searchByClientId(detail.client_id)}
                    className="px-2 py-1 rounded border border-primary/50 bg-primary/10 text-primary text-xs hover:bg-primary/20 transition-colors"
                  >
                    Buscar por este client_id
                  </button>
                </span>
              </dd>
              <dt className="text-muted-foreground">provider</dt>
              <dd>{detail.provider}</dd>
              <dt className="text-muted-foreground">model</dt>
              <dd>{detail.model ?? "—"}</dd>
              <dt className="text-muted-foreground">status</dt>
              <dd>
                <Badge variant={detail.status === "ok" ? "success" : "danger"}>{detail.status}</Badge>
              </dd>
              <dt className="text-muted-foreground">prompt_tokens</dt>
              <dd>{detail.prompt_tokens ?? "—"}</dd>
              <dt className="text-muted-foreground">completion_tokens</dt>
              <dd>{detail.completion_tokens ?? "—"}</dd>
              <dt className="text-muted-foreground">latency_ms</dt>
              <dd>{detail.latency_ms ?? "—"}</dd>
              <dt className="text-muted-foreground">input_size</dt>
              <dd>{detail.input_size ?? "—"}</dd>
              <dt className="text-muted-foreground">output_size</dt>
              <dd>{detail.output_size ?? "—"}</dd>
            </dl>
          </div>
        )}
      </Drawer>
    </div>
  );
}
