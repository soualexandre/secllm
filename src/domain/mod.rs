//! Domain core – entities and rules, no external infrastructure deps.

pub mod models;
pub mod pii;
pub mod governance;

pub use models::{AuditEvent, RequestContext, MaskedSpan};
pub use pii::{PiiKind, PiiMatch};
pub use governance::GovernancePolicy;
