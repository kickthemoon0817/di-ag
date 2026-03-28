use super::{detect_format, parse_input, read_input};
use di_ag_validate::Severity;

pub fn run(input: &str, strict: bool, json: bool) -> Result<(), String> {
    let content = read_input(input)?;
    let fmt = if input == "-" {
        "dsl"
    } else {
        detect_format(input)
    };
    let doc = parse_input(&content, fmt)?;

    let report = di_ag_validate::validate(&doc);

    let has_errors = report
        .violations
        .iter()
        .any(|v| v.severity == Severity::Error);
    let has_warnings = report
        .violations
        .iter()
        .any(|v| v.severity == Severity::Warn);
    let fail = has_errors || (strict && has_warnings);

    if json {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else if report.violations.is_empty() {
        eprintln!("Valid: no issues found.");
    } else {
        for v in &report.violations {
            let severity = match v.severity {
                Severity::Error => "ERROR",
                Severity::Warn => "WARN",
                Severity::Info => "INFO",
            };
            eprintln!(
                "[{}] {}: {}",
                severity,
                v.violation_type,
                v.message.as_deref().unwrap_or("")
            );
        }
    }

    if fail {
        std::process::exit(1);
    }
    Ok(())
}
