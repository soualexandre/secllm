import axios from "axios";

// No browser: chamadas passam pelo proxy Next.js para enviar o token do cookie ao backend
const baseURL =
  typeof window !== "undefined"
    ? "/api/backend"
    : (process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:3010");

export const api = axios.create({
  baseURL,
  headers: { "Content-Type": "application/json" },
  withCredentials: true,
});

api.interceptors.request.use((config) => {
  if (typeof window === "undefined") return config;
  const token = document.cookie
    .split("; ")
    .find((row) => row.startsWith("secllm_token="))
    ?.split("=")[1];
  if (token) config.headers.Authorization = `Bearer ${token}`;
  return config;
});

api.interceptors.response.use(
  (res) => res,
  (err) => {
    if (err.response?.status === 401) {
      if (typeof window !== "undefined") {
        document.cookie = "secllm_token=; path=/; max-age=0";
        window.location.href = "/login";
      }
    }
    return Promise.reject(err);
  }
);
