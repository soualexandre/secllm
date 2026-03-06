import { NextRequest, NextResponse } from "next/server";

const API_URL = (process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:3010").replace(/\/$/, "");

export async function GET(request: NextRequest, { params }: { params: Promise<{ path: string[] }> }) {
  return proxy(request, await params, "GET");
}

export async function POST(request: NextRequest, { params }: { params: Promise<{ path: string[] }> }) {
  return proxy(request, await params, "POST");
}

export async function PUT(request: NextRequest, { params }: { params: Promise<{ path: string[] }> }) {
  return proxy(request, await params, "PUT");
}

export async function DELETE(request: NextRequest, { params }: { params: Promise<{ path: string[] }> }) {
  return proxy(request, await params, "DELETE");
}

async function proxy(
  request: NextRequest,
  { path }: { path: string[] },
  method: string
): Promise<NextResponse> {
  const pathname = path.join("/");
  const search = request.nextUrl.searchParams.toString();
  const url = search ? `${API_URL}/${pathname}?${search}` : `${API_URL}/${pathname}`;
  const token = request.cookies.get("secllm_token")?.value;

  const headers: Record<string, string> = {
    "Content-Type": "application/json",
  };
  if (token) headers.Authorization = `Bearer ${token}`;

  let body: string | undefined;
  try {
    if (method !== "GET" && request.body) {
      body = await request.text();
    }
  } catch {
    return NextResponse.json({ error: "Invalid body" }, { status: 400 });
  }

  const res = await fetch(url, {
    method,
    headers,
    body: body ?? undefined,
  });

  const text = await res.text();
  const status = res.status;

  // 204 No Content / 205 Reset Content: body may be empty or absent; do not parse as JSON
  if (status === 204 || status === 205) {
    return new NextResponse(null, { status });
  }

  const trimmed = text.trim();
  let data: unknown;
  try {
    data = trimmed ? JSON.parse(trimmed) : null;
  } catch {
    const snippet = trimmed.length > 120 ? `${trimmed.slice(0, 120)}…` : trimmed;
    return NextResponse.json(
      {
        error: "Invalid JSON from backend",
        backendStatus: status,
        snippet: snippet || "(empty body)",
      },
      { status: 502 }
    );
  }

  return NextResponse.json(data, { status });
}
