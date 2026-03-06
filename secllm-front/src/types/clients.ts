export interface Client {
  client_id: string;
  name?: string;
  created_at?: string;
  keys?: string[];
  has_secret?: boolean;
}

/** Item retornado por GET /api/v1/clients (listagem do Postgres) */
export interface ListClientItem {
  client_id: string;
  name?: string;
  keys: string[];
  has_secret: boolean;
}

export interface CreateClientRequest {
  name?: string;
}

export interface CreateClientResponse {
  client_id: string;
  client_secret: string;
  name?: string;
}

/** Resposta de GET /api/v1/clients/:id/credentials */
export interface ClientCredentials {
  keys: Record<string, string | null>;
  client_secret?: string | null;
}
