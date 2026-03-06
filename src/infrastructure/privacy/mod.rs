//! Privacy adapter – PII/secret detection and masking (PrivacyPort).

mod detector;
mod masker;

use crate::domain::{GovernancePolicy, MaskedSpan, PiiMatch};
use crate::Result;

pub use detector::PiiDetector;
pub use masker::apply_masks;

/// Privacy service: detect PII and apply masks according to policy.
pub struct PrivacyService {
    detector: PiiDetector,
    policy: GovernancePolicy,
}

impl PrivacyService {
    pub fn new(policy: GovernancePolicy) -> Self {
        Self {
            detector: PiiDetector::default(),
            policy,
        }
    }

    pub fn scan_and_mask_impl(&self, text: &str) -> Result<(String, Vec<MaskedSpan>)> {
        self.scan_and_mask_with_policy_impl(text, &self.policy)
    }

    pub fn scan_and_mask_with_policy_impl(
        &self,
        text: &str,
        policy: &GovernancePolicy,
    ) -> Result<(String, Vec<MaskedSpan>)> {
        let matches = self.detector.detect(text);
        let filtered: Vec<PiiMatch> = matches
            .into_iter()
            .filter(|m| policy.should_mask(m.kind.clone()))
            .collect();
        let masked = apply_masks(text, &filtered);
        let spans: Vec<MaskedSpan> = filtered
            .iter()
            .map(|m| MaskedSpan {
                start: m.start,
                end: m.end,
                kind: format!("{:?}", m.kind),
                replacement: m.replacement(),
            })
            .collect();
        Ok((masked, spans))
    }

    pub fn detect_with_policy_impl(
        &self,
        text: &str,
        policy: &GovernancePolicy,
    ) -> Result<Vec<MaskedSpan>> {
        let matches = self.detector.detect(text);
        let filtered: Vec<PiiMatch> = matches
            .into_iter()
            .filter(|m| policy.should_mask(m.kind.clone()))
            .collect();
        let spans: Vec<MaskedSpan> = filtered
            .iter()
            .map(|m| MaskedSpan {
                start: m.start,
                end: m.end,
                kind: format!("{:?}", m.kind),
                replacement: m.replacement(),
            })
            .collect();
        Ok(spans)
    }
}

impl crate::application::ports::PrivacyPort for PrivacyService {
    fn scan_and_mask(&self, text: &str) -> Result<(String, Vec<MaskedSpan>)> {
        self.scan_and_mask_impl(text)
    }

    fn scan_and_mask_with_policy(
        &self,
        text: &str,
        policy: &GovernancePolicy,
    ) -> Result<(String, Vec<MaskedSpan>)> {
        self.scan_and_mask_with_policy_impl(text, policy)
    }

    fn detect_with_policy(
        &self,
        text: &str,
        policy: &GovernancePolicy,
    ) -> Result<Vec<MaskedSpan>> {
        self.detect_with_policy_impl(text, policy)
    }
}
