//! Governance policies (allow/deny by type); can be extended later.

use crate::domain::PiiKind;

#[derive(Clone, Debug, Default)]
pub struct GovernancePolicy {
    /// PII kinds that must be masked before sending to LLM.
    pub mask_pii: Vec<PiiKind>,
    /// Whether to scan and mask LLM response as well.
    pub mask_response: bool,
}

impl GovernancePolicy {
    pub fn default_strict() -> Self {
        Self {
            mask_pii: vec![
                PiiKind::Cpf,
                PiiKind::Name,
                PiiKind::Email,
                PiiKind::Phone,
                PiiKind::Secret,
            ],
            mask_response: true,
        }
    }

    pub fn should_mask(&self, kind: PiiKind) -> bool {
        self.mask_pii.contains(&kind) || matches!(kind, PiiKind::Secret)
    }
}
