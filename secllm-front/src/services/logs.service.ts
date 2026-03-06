import { api } from "./api-client";
import type { LogsResponse, MetricsResponse } from "@/types/logs";

export interface LogsParams {
  limit?: number;
  offset?: number;
  client_id?: string;
  provider?: string;
  status?: string;
  sort?: string;
  order?: "asc" | "desc";
}

export async function getLogs(params?: LogsParams): Promise<LogsResponse> {
  const p = params ?? {};
  const query: Record<string, string | number | undefined> = {
    limit: p.limit,
    offset: p.offset,
    client_id: p.client_id?.trim() || undefined,
    provider: p.provider?.trim() || undefined,
    status: p.status?.trim() || undefined,
    sort: p.sort?.trim() || undefined,
    order: p.order,
  };
  const filtered = Object.fromEntries(Object.entries(query).filter(([, v]) => v != null && v !== ""));
  const { data } = await api.get<LogsResponse>("/api/v1/logs", { params: filtered });
  return data;
}

export interface MetricsParams {
  provider?: string;
  status?: string;
}

export async function getMetrics(params?: MetricsParams): Promise<MetricsResponse> {
  const filtered = params
    ? Object.fromEntries(
        Object.entries({ provider: params.provider?.trim(), status: params.status?.trim() }).filter(
          ([, v]) => v != null && v !== ""
        )
      )
    : {};
  const { data } = await api.get<MetricsResponse>("/api/v1/metrics", {
    params: Object.keys(filtered).length ? filtered : undefined,
  });
  return data;
}
