//! PII types and detection rules (domain rules only; regex/impl in infrastructure).

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PiiKind {
    Cpf,
    Name,
    Email,
    Phone,
    Secret,
    Custom(String),
}

/// A single PII match in text (offsets and kind).
#[derive(Clone, Debug)]
pub struct PiiMatch {
    pub start: usize,
    pub end: usize,
    pub kind: PiiKind,
    pub raw: String,
}

impl PiiMatch {
    pub fn replacement(&self) -> String {
        match self.kind {
            PiiKind::Cpf => "***.***.***-**".to_string(),
            PiiKind::Name => "[NOME]".to_string(),
            PiiKind::Email => "***@***.***".to_string(),
            PiiKind::Phone => "****-****".to_string(),
            PiiKind::Secret => "***SECRET***".to_string(),
            PiiKind::Custom(_) => "***".to_string(),
        }
    }
}
