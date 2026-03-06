# SecLLM Frontend – AI Governance Gateway Dashboard

Next.js 14+ dashboard for SecLLM: governance, clients (vault), billing, and monitoring.

## Stack

- Next.js 16 (App Router)
- React 19, TypeScript
- Tailwind CSS
- TanStack Query, Zustand, Zod, Axios
- cmdk (command palette)

## Setup

```bash
npm install
cp .env.example .env.local
```

Set in `.env.local`:

- `NEXT_PUBLIC_API_URL=http://localhost:3010` (SecLLM backend)

## Scripts

- `npm run dev` – development server
- `npm run build` – production build
- `npm run start` – run production build
- `npm test` – unit tests (Jest)
- `npm run test:e2e` – E2E tests (Playwright)

## Deploy

### Vercel

1. Connect the repo to Vercel.
2. Set env var `NEXT_PUBLIC_API_URL` to your SecLLM API URL.
3. Deploy.

### Docker

Build and run with the included Dockerfile (standalone output):

```bash
docker build -t secllm-front .
docker run -p 3000:3000 -e NEXT_PUBLIC_API_URL=http://backend:3010 secllm-front
```

## Flow

1. **Register** at `/register`, then **Sign in** at `/login`.
2. Use **Dashboard** for overview, **Clients** to create clients and manage API keys, **Governance** to edit global/client policies (JSON).
3. **Cmd+K** (or Ctrl+K) opens the command palette for quick navigation.
