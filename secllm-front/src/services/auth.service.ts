import { api } from "./api-client";
import type { AuthTokenRequest, AuthTokenResponse, RegisterRequest, RegisterResponse } from "@/types";

export async function login(body: AuthTokenRequest): Promise<AuthTokenResponse> {
  const { data } = await api.post<AuthTokenResponse>("/auth/token", body);
  return data;
}

export async function register(body: RegisterRequest): Promise<RegisterResponse> {
  const { data } = await api.post<RegisterResponse>("/auth/register", body);
  return data;
}
