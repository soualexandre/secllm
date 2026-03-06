import { api } from "./api-client";

export interface BillingLogInput {
  period_start: string;
  period_end: string;
  amount_cents: number;
  details?: Record<string, unknown>;
  client_id?: string;
}

export async function postBillingLog(body: BillingLogInput): Promise<void> {
  await api.post("/api/v1/billing/logs", body);
}
