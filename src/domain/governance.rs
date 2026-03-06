//! Governance policies (allow/deny by type); can be extended later.

use crate::domain::PiiKind;
use serde_json::Value;

#[derive(Clone, Debug, Default)]
pub struct GovernancePolicy {
    /// PII kinds that must be masked before sending to LLM.
    pub mask_pii: Vec<PiiKind>,
    /// Whether to scan and mask LLM response as well.
    pub mask_response: bool,
    /// If true, reject request with 400 when PII is detected instead of masking.
    pub block_on_pii: bool,
}

impl GovernancePolicy {
    pub fn default_strict() -> Self {
        Self {
            mask_pii: vec![
                PiiKind::Cpf,
                PiiKind::Rg,
                PiiKind::Cnpj,
                PiiKind::Name,
                PiiKind::Email,
                PiiKind::Phone,
                PiiKind::Secret,
            ],
            mask_response: true,
            block_on_pii: false,
        }
    }

    pub fn should_mask(&self, kind: PiiKind) -> bool {
        self.mask_pii.contains(&kind) || matches!(kind, PiiKind::Secret)
    }

    /// Parse policy from JSON (e.g. from governance_policies table).
    /// mask_pii: array of strings like "Cpf", "Rg", "Email". block_on_pii: optional bool.
    pub fn from_json_value(v: &Value) -> Self {
        let mask_pii = v
            .get("mask_pii")
            .and_then(|a| a.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|s| s.as_str())
                    .filter_map(|s| parse_pii_kind(s))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let mask_response = v
            .get("mask_response")
            .and_then(|b| b.as_bool())
            .unwrap_or(true);
        let block_on_pii = v
            .get("block_on_pii")
            .and_then(|b| b.as_bool())
            .unwrap_or(false);
        Self {
            mask_pii,
            mask_response,
            block_on_pii,
        }
    }
}

fn parse_pii_kind(s: &str) -> Option<PiiKind> {
    match s.trim() {
        "Cpf" => Some(PiiKind::Cpf),
        "Rg" => Some(PiiKind::Rg),
        "Cnpj" => Some(PiiKind::Cnpj),
        "Name" => Some(PiiKind::Name),
        "Email" => Some(PiiKind::Email),
        "Phone" => Some(PiiKind::Phone),
        "Secret" => Some(PiiKind::Secret),
        _ if !s.is_empty() => Some(PiiKind::Custom(s.to_string())),
        _ => None,
    }
}
