import { api } from "./api-client";
import type { GovernancePolicy } from "@/types";

export async function getGovernanceGlobal(): Promise<GovernancePolicy> {
  const { data } = await api.get<GovernancePolicy>("/api/v1/governance/global");
  return data;
}

export async function putGovernanceGlobal(policy: GovernancePolicy): Promise<void> {
  await api.put("/api/v1/governance/global", { policy });
}

export async function getGovernanceClient(clientId: string): Promise<GovernancePolicy> {
  const { data } = await api.get<GovernancePolicy>(`/api/v1/governance/clients/${clientId}`);
  return data;
}

export async function putGovernanceClient(clientId: string, policy: GovernancePolicy): Promise<void> {
  await api.put(`/api/v1/governance/clients/${clientId}`, { policy });
}
