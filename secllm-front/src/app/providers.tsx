"use client";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState, useEffect } from "react";
import { themeStore } from "@/stores/theme.store";

export function Providers({ children }: { children: React.ReactNode }) {
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: { staleTime: 60_000, gcTime: 5 * 60_000 },
        },
      })
  );

  useEffect(() => {
    themeStore.init();
  }, []);

  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
}
