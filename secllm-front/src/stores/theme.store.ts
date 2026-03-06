import { useEffect, useState } from "react";

const STORAGE_KEY = "secllm_theme";

export type Theme = "light" | "dark" | "system";

function getStored(): Theme {
  if (typeof window === "undefined") return "system";
  const v = window.localStorage.getItem(STORAGE_KEY);
  if (v === "light" || v === "dark" || v === "system") return v;
  return "system";
}

function getEffectiveTheme(theme: Theme): "light" | "dark" {
  if (theme === "light") return "light";
  if (theme === "dark") return "dark";
  if (typeof window !== "undefined" && window.matchMedia("(prefers-color-scheme: dark)").matches) {
    return "dark";
  }
  return "light";
}

export function applyTheme(theme: Theme) {
  const effective = theme === "system" ? getEffectiveTheme("system") : theme;
  const root = typeof document !== "undefined" ? document.documentElement : null;
  if (root) {
    root.classList.toggle("dark", effective === "dark");
  }
}

const listeners = new Set<() => void>();
let theme: Theme = "system";

function getTheme(): Theme {
  return theme;
}

function setTheme(next: Theme) {
  theme = next;
  if (typeof window !== "undefined") {
    window.localStorage.setItem(STORAGE_KEY, next);
    applyTheme(next);
  }
  listeners.forEach((l) => l());
}

function subscribe(listener: () => void) {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

let mediaQuery: MediaQueryList | null = null;

function init() {
  theme = getStored();
  applyTheme(theme);
  if (typeof window !== "undefined" && !mediaQuery) {
    mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    mediaQuery.addEventListener("change", () => {
      if (theme === "system") applyTheme("system");
    });
  }
}

export const themeStore = { getTheme, setTheme, subscribe, init };

export function useTheme(): { theme: Theme; setTheme: (t: Theme) => void } {
  const [value, setValue] = useState<Theme>("system");

  useEffect(() => {
    themeStore.init();
    setValue(themeStore.getTheme());
    const unsub = themeStore.subscribe(() => setValue(themeStore.getTheme()));
    return unsub;
  }, []);

  return {
    theme: value,
    setTheme: themeStore.setTheme,
  };
}
