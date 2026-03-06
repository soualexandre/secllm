import { create } from "zustand";
import { persist } from "zustand/middleware";

export interface User {
  id?: string;
  sub?: string;
  email?: string;
  name?: string | null;
  role?: string;
}

interface AuthState {
  user: User | null;
  isAuthenticated: boolean;
  setAuth: (user: User | null) => void;
  logout: () => void;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set) => ({
      user: null,
      isAuthenticated: false,
      setAuth: (user) => set({ user, isAuthenticated: !!user }),
      logout: () => set({ user: null, isAuthenticated: false }),
    }),
    { name: "secllm-auth", partialize: (s) => ({ isAuthenticated: s.isAuthenticated, user: s.user }) }
  )
);
