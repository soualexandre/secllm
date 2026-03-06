"use client";

import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  getGovernanceGlobal,
  putGovernanceGlobal,
  getGovernanceClient,
  putGovernanceClient,
} from "@/services/governance.service";
import type { GovernancePolicy } from "@/types";

export function useGovernanceGlobal() {
  return useQuery({
    queryKey: ["governance", "global"],
    queryFn: getGovernanceGlobal,
  });
}

export function usePutGovernanceGlobal() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (policy: GovernancePolicy) => putGovernanceGlobal(policy),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["governance", "global"] }),
  });
}

export function useGovernanceClient(clientId: string | null) {
  return useQuery({
    queryKey: ["governance", "client", clientId],
    queryFn: () => getGovernanceClient(clientId!),
    enabled: !!clientId,
  });
}

export function usePutGovernanceClient(clientId: string | null) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (policy: GovernancePolicy) => putGovernanceClient(clientId!, policy),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["governance", "client", clientId] }),
  });
}
