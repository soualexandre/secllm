export interface GovernancePolicy {
  mask_pii?: string[];
  mask_response?: boolean;
  rate_limits?: Record<string, unknown>;
  allowed_models?: string[];
  blocked_terms?: string[];
  data_redaction?: Record<string, unknown>;
  [key: string]: unknown;
}
