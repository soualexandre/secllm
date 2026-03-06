export interface MeResponse {
  id: string;
  email: string;
  name?: string | null;
  role: string;
}

export interface AuthTokenRequest {
  email?: string;
  password?: string;
  client_id?: string;
  client_secret?: string;
  provider?: string;
}

export interface AuthTokenResponse {
  access_token: string;
  token_type: string;
  expires_in: number;
}

export interface RegisterRequest {
  email: string;
  password: string;
  name?: string;
}

export interface RegisterResponse {
  id: string;
  email: string;
  name?: string;
}
