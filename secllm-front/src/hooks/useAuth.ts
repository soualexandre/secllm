"use client";

import { useCallback, useEffect } from "react";
import { useAuthStore } from "@/stores/auth.store";
import { loginSchema, registerSchema } from "@/lib/validators";
import type { LoginInput, RegisterInput } from "@/lib/validators";

export function useAuth() {
  const { user, isAuthenticated, setAuth, logout } = useAuthStore();

  useEffect(() => {
    if (!isAuthenticated || user?.email) return;
    fetch("/api/auth/me", { credentials: "include" })
      .then((r) => (r.ok ? r.json() : null))
      .then((data) => {
        if (data?.email) setAuth({ id: data.id, email: data.email, name: data.name ?? null, role: data.role });
      })
      .catch(() => {});
  }, [isAuthenticated, user?.email, setAuth]);

  const login = useCallback(async (input: LoginInput) => {
    const parsed = loginSchema.safeParse(input);
    if (!parsed.success) throw new Error(parsed.error.issues[0]?.message ?? "Validation failed");
    const res = await fetch("/api/auth/login", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(parsed.data),
    });
    const data = await res.json();
    if (!res.ok) throw new Error(data.error ?? "Login failed");
    if (data.user) {
      setAuth({
        id: data.user.id,
        email: data.user.email,
        name: data.user.name ?? null,
        role: data.user.role,
      });
    } else {
      setAuth({ sub: data.access_token ? "user" : undefined });
    }
    return data;
  }, [setAuth]);

  const registerUser = useCallback(async (input: RegisterInput) => {
    const parsed = registerSchema.safeParse(input);
    if (!parsed.success) throw new Error(parsed.error.issues[0]?.message ?? "Validation failed");
    const API_URL = process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:3010";
    const res = await fetch(`${API_URL}/auth/register`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(parsed.data),
    });
    const data = await res.json();
    if (!res.ok) throw new Error(data.error ?? "Registration failed");
    return data;
  }, []);

  const logoutUser = useCallback(async () => {
    await fetch("/api/auth/logout", { method: "POST" });
    logout();
  }, [logout]);

  return { user, isAuthenticated, login, register: registerUser, logout: logoutUser };
}
