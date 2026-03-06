import { api } from "./api-client";
import type {
  CreateClientRequest,
  CreateClientResponse,
  ListClientItem,
  ClientCredentials,
} from "@/types";

export async function getClients(): Promise<ListClientItem[]> {
  const { data } = await api.get<ListClientItem[]>("/api/v1/clients");
  return data;
}

export async function getProviders(): Promise<string[]> {
  const { data } = await api.get<string[]>("/api/v1/providers");
  return data;
}

export async function getCredentials(clientId: string): Promise<ClientCredentials> {
  const { data } = await api.get<ClientCredentials>(
    `/api/v1/clients/${clientId}/credentials`
  );
  return data;
}

export async function createClient(body: CreateClientRequest): Promise<CreateClientResponse> {
  const { data } = await api.post<CreateClientResponse>("/api/v1/clients", body);
  return data;
}

export async function putApiKey(clientId: string, provider: string, apiKey: string): Promise<void> {
  await api.put(`/api/v1/clients/${clientId}/keys/${provider}`, { api_key: apiKey });
}

export async function deleteApiKey(clientId: string, provider: string): Promise<void> {
  await api.delete(`/api/v1/clients/${clientId}/keys/${provider}`);
}

export async function putClientSecret(clientId: string, clientSecret: string): Promise<void> {
  await api.put(`/api/v1/clients/${clientId}/secret`, { client_secret: clientSecret });
}

export async function deleteClientSecret(clientId: string): Promise<void> {
  await api.delete(`/api/v1/clients/${clientId}/secret`);
}
