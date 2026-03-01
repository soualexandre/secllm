//! Apply masks to text given PII matches (reverse order to preserve offsets).

use crate::domain::PiiMatch;

pub fn apply_masks(text: &str, matches: &[PiiMatch]) -> String {
    let mut out = text.to_string();
    for m in matches.iter().rev() {
        let replacement = m.replacement();
        out.replace_range(m.start..m.end, &replacement);
    }
    out
}
