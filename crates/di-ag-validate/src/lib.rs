pub mod report;
pub mod rules;

#[cfg(test)]
mod tests;

use di_ag_ir::Document;
pub use report::{Severity, ValidationReport, Violation};

pub fn validate(doc: &Document) -> ValidationReport {
    let mut violations = vec![];

    violations.extend(rules::check_duplicate_ids(doc));
    violations.extend(rules::check_edge_references(doc));
    violations.extend(rules::check_label_lengths(doc));
    violations.extend(rules::check_orphan_nodes(doc));
    violations.extend(rules::check_self_loops(doc));
    violations.extend(rules::check_cycles(doc));
    violations.extend(rules::check_duplicate_edge_ids(doc));

    let valid = !violations.iter().any(|v| v.severity == Severity::Error);

    ValidationReport { valid, violations }
}
