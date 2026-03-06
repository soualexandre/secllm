"use client";

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  getClients,
  getProviders,
  createClient,
  putApiKey,
  deleteApiKey,
  putClientSecret,
  deleteClientSecret,
} from "@/services/clients.service";
import type { CreateClientRequest } from "@/types";

export const clientsQueryKey = ["clients"] as const;
export const providersQueryKey = ["providers"] as const;

export function useClientsList() {
  return useQuery({
    queryKey: clientsQueryKey,
    queryFn: getClients,
  });
}

export function useProvidersList() {
  return useQuery({
    queryKey: providersQueryKey,
    queryFn: getProviders,
  });
}

export function useCreateClient() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: CreateClientRequest) => createClient(body),
    onSuccess: () => qc.invalidateQueries({ queryKey: clientsQueryKey }),
  });
}

export function usePutApiKey() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ clientId, provider, apiKey }: { clientId: string; provider: string; apiKey: string }) =>
      putApiKey(clientId, provider, apiKey),
    onSuccess: () => qc.invalidateQueries({ queryKey: clientsQueryKey }),
  });
}

export function useDeleteApiKey() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ clientId, provider }: { clientId: string; provider: string }) =>
      deleteApiKey(clientId, provider),
    onSuccess: () => qc.invalidateQueries({ queryKey: clientsQueryKey }),
  });
}

export function usePutClientSecret() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ clientId, clientSecret }: { clientId: string; clientSecret: string }) =>
      putClientSecret(clientId, clientSecret),
    onSuccess: () => qc.invalidateQueries({ queryKey: clientsQueryKey }),
  });
}

export function useDeleteClientSecret() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (clientId: string) => deleteClientSecret(clientId),
    onSuccess: () => qc.invalidateQueries({ queryKey: clientsQueryKey }),
  });
}
