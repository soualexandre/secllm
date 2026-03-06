"use client";

import { useCallback, useEffect, useState } from "react";
import { Card } from "@/components/ui";
import { getMetrics } from "@/services/logs.service";
import type { MetricsResponse } from "@/types/logs";

export default function DashboardPage() {
  const [metrics, setMetrics] = useState<MetricsResponse | null>(null);
  /** Listas sempre completas (sem filtro) para não sumir itens ao filtrar */
  const [listMetrics, setListMetrics] = useState<MetricsResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [filterProvider, setFilterProvider] = useState<string>("");
  const [filterStatus, setFilterStatus] = useState<string>("");

  const fetchMetrics = useCallback(() => {
    setLoading(true);
    setError(null);
    const params =
      filterProvider || filterStatus
        ? { provider: filterProvider || undefined, status: filterStatus || undefined }
        : undefined;
    getMetrics(params)
      .then(setMetrics)
      .catch((e) => setError(e instanceof Error ? e.message : "Erro ao carregar métricas"))
      .finally(() => setLoading(false));
  }, [filterProvider, filterStatus]);

  /** Carrega listas completas uma vez (e ao limpar filtros) para manter todos os itens visíveis */
  const fetchListMetrics = useCallback(() => {
    getMetrics()
      .then(setListMetrics)
      .catch(() => {});
  }, []);

  useEffect(() => {
    fetchMetrics();
  }, [fetchMetrics]);

  useEffect(() => {
    if (!filterProvider && !filterStatus) {
      setListMetrics(metrics);
    } else if (!listMetrics) {
      fetchListMetrics();
    }
  }, [filterProvider, filterStatus, metrics, listMetrics, fetchListMetrics]);

  const hasFilters = !!filterProvider || !!filterStatus;
  const clearFilters = () => {
    setFilterProvider("");
    setFilterStatus("");
  };

  if (loading && !metrics) {
    return (
      <div className="space-y-6">
        <h2 className="text-2xl font-semibold">Dashboard</h2>
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
          {[1, 2, 3, 4].map((i) => (
            <Card key={i}>
              <div className="h-4 w-20 bg-muted rounded animate-pulse mb-2" />
              <div className="h-8 w-16 bg-muted rounded animate-pulse" />
            </Card>
          ))}
        </div>
        <Card title="Por provider">
          <div className="h-24 bg-muted rounded animate-pulse" />
        </Card>
      </div>
    );
  }

  if (error && !metrics) {
    return (
      <div className="space-y-6">
        <h2 className="text-2xl font-semibold">Dashboard</h2>
        <Card>
          <p className="text-danger">{error}</p>
          <p className="text-sm text-muted-foreground mt-2">
            Verifique se o backend e o ClickHouse estão rodando e se você está autenticado.
          </p>
        </Card>
      </div>
    );
  }

  const m = metrics ?? ({} as MetricsResponse);
  const errorRate =
    m.total_requests > 0 ? ((m.error_count / m.total_requests) * 100).toFixed(1) : "0";
  const avgLatency =
    m.avg_latency_ms != null ? `${Math.round(m.avg_latency_ms)} ms` : "—";
  const minLatency =
    m.min_latency_ms != null ? `${Math.round(m.min_latency_ms)} ms` : "—";
  const maxLatency =
    m.max_latency_ms != null ? `${Math.round(m.max_latency_ms)} ms` : "—";
  const totalTokens = m.total_prompt_tokens + m.total_completion_tokens;

  const lists = listMetrics ?? metrics ?? ({} as MetricsResponse);
  const byProvider = lists.by_provider ?? [];
  const byStatus = lists.by_status ?? [];

  return (
    <div className="space-y-6">
      <div className="flex flex-nowrap items-center justify-between gap-4">
        <h2 className="text-2xl font-semibold">Dashboard</h2>
        {hasFilters && (
          <div className="flex flex-shrink-0 items-center gap-2">
            {filterProvider && (
              <span className="inline-flex items-center gap-1 rounded-md bg-blue-50 px-2 py-1 text-xs text-blue-700 dark:bg-blue-950/50 dark:text-blue-300">
                {filterProvider}
                <button
                  type="button"
                  onClick={() => setFilterProvider("")}
                  className="rounded p-0.5 hover:bg-blue-200/50 dark:hover:bg-blue-800/40"
                  aria-label="Remover filtro provider"
                >
                  ×
                </button>
              </span>
            )}
            {filterStatus && (
              <span className="inline-flex items-center gap-1 rounded-md bg-blue-50 px-2 py-1 text-xs text-blue-700 dark:bg-blue-950/50 dark:text-blue-300">
                {filterStatus}
                <button
                  type="button"
                  onClick={() => setFilterStatus("")}
                  className="rounded p-0.5 hover:bg-blue-200/50 dark:hover:bg-blue-800/40"
                  aria-label="Remover filtro status"
                >
                  ×
                </button>
              </span>
            )}
            <button
              type="button"
              onClick={clearFilters}
              className="text-xs text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-300"
            >
              Limpar filtros
            </button>
          </div>
        )}
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <Card>
          <p className="text-sm text-muted-foreground">Total de requisições</p>
          <p className="text-2xl font-semibold">{(m.total_requests ?? 0).toLocaleString()}</p>
        </Card>
        <Card>
          <p className="text-sm text-muted-foreground">Tokens (prompt + completion)</p>
          <p className="text-2xl font-semibold">{totalTokens.toLocaleString()}</p>
        </Card>
        <Card>
          <p className="text-sm text-muted-foreground">Latência média</p>
          <p className="text-2xl font-semibold">{avgLatency}</p>
        </Card>
        <Card>
          <p className="text-sm text-muted-foreground">Taxa de erro</p>
          <p className="text-2xl font-semibold">{errorRate}%</p>
        </Card>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        <Card title="Latência (min / max)">
          <p className="text-sm text-muted-foreground">
            Mín: {minLatency} — Máx: {maxLatency}
          </p>
        </Card>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        <Card title="Por provider">
          {byProvider.length === 0 ? (
            <p className="text-muted-foreground text-sm">Nenhum dado ainda.</p>
          ) : (
            <ul className="space-y-1">
              {byProvider.map(({ provider, count }) => (
                <li key={provider}>
                  <button
                    type="button"
                    onClick={() =>
                      setFilterProvider((p) => (p === provider ? "" : provider))
                    }
                    className={`flex w-full items-center justify-between rounded px-3 py-2 text-left text-sm transition-colors hover:bg-muted/40 ${
                      filterProvider === provider
                        ? "bg-primary/15 ring-1 ring-primary font-medium"
                        : ""
                    }`}
                  >
                    <span>{provider}</span>
                    <span className="text-muted-foreground">{count.toLocaleString()}</span>
                  </button>
                </li>
              ))}
            </ul>
          )}
        </Card>
        <Card title="Por status">
          {byStatus.length === 0 ? (
            <p className="text-muted-foreground text-sm">Nenhum dado ainda.</p>
          ) : (
            <ul className="space-y-1">
              {byStatus.map(({ status, count }) => (
                <li key={status}>
                  <button
                    type="button"
                    onClick={() =>
                      setFilterStatus((s) => (s === status ? "" : status))
                    }
                    className={`flex w-full items-center justify-between rounded px-3 py-2 text-left text-sm transition-colors hover:bg-muted/40 ${
                      filterStatus === status
                        ? "bg-primary/15 ring-1 ring-primary font-medium"
                        : ""
                    }`}
                  >
                    <span>{status}</span>
                    <span className="text-muted-foreground">{count.toLocaleString()}</span>
                  </button>
                </li>
              ))}
            </ul>
          )}
        </Card>
      </div>
    </div>
  );
}
