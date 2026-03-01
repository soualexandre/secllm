//! PII and secret detection via regex and heuristics.

use regex::Regex;
use crate::domain::{PiiKind, PiiMatch};
use std::sync::LazyLock;

static CPF_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b\d{3}\.?\d{3}\.?\d{3}-?\d{2}\b").unwrap()
});
static EMAIL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap()
});
static PHONE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\(?\d{2}\)?\s?\d{4,5}-?\d{4}\b").unwrap()
});
static SECRET_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(api[_-]?key|secret|password|token)\s*[:=]\s*['\"]?[\w-]{8,}['\"]?").unwrap()
});

#[derive(Default)]
pub struct PiiDetector;

impl PiiDetector {
    pub fn detect(&self, text: &str) -> Vec<PiiMatch> {
        let mut out = Vec::new();
        for m in CPF_RE.find_iter(text) {
            out.push(PiiMatch {
                start: m.start(),
                end: m.end(),
                kind: PiiKind::Cpf,
                raw: m.as_str().to_string(),
            });
        }
        for m in EMAIL_RE.find_iter(text) {
            out.push(PiiMatch {
                start: m.start(),
                end: m.end(),
                kind: PiiKind::Email,
                raw: m.as_str().to_string(),
            });
        }
        for m in PHONE_RE.find_iter(text) {
            out.push(PiiMatch {
                start: m.start(),
                end: m.end(),
                kind: PiiKind::Phone,
                raw: m.as_str().to_string(),
            });
        }
        for m in SECRET_RE.find_iter(text) {
            out.push(PiiMatch {
                start: m.start(),
                end: m.end(),
                kind: PiiKind::Secret,
                raw: m.as_str().to_string(),
            });
        }
        sort_and_dedup(&mut out);
        out
    }
}

fn sort_and_dedup(matches: &mut Vec<PiiMatch>) {
    matches.sort_by_key(|m| m.start);
    matches.dedup_by_key(|m| m.start);
}
