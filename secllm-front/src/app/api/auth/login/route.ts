import { NextRequest, NextResponse } from "next/server";

const API_URL = (process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:3010").replace(/\/$/, "");

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const res = await fetch(`${API_URL}/auth/token`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    });
    const data = await res.json();
    if (!res.ok) {
      return NextResponse.json(data, { status: res.status });
    }
    const token = data.access_token as string;
    let user: { id: string; email: string; name?: string | null; role: string } | null = null;
    try {
      const meRes = await fetch(`${API_URL}/api/v1/me`, {
        headers: { Authorization: `Bearer ${token}` },
      });
      if (meRes.ok) {
        user = await meRes.json();
      }
    } catch {
      // ignore; frontend can fetch /api/auth/me later
    }
    const response = NextResponse.json({
      access_token: token,
      token_type: data.token_type,
      expires_in: data.expires_in,
      user,
    });
    response.cookies.set("secllm_token", token, {
      secure: process.env.NODE_ENV === "production",
      sameSite: "strict",
      maxAge: 60 * 60 * 24,
      path: "/",
    });
    return response;
  } catch (e) {
    return NextResponse.json({ error: "Login failed" }, { status: 500 });
  }
}
