//! Human rendering for optional registry and package evidence.

use crate::{
    redact_report_value, PackageOptionalEvidence, PackagePerson, RegistryEvidence, RegistryPerson,
};

/// Render optional registry metadata when present.
pub(crate) fn render_registry_optional_evidence(registry: &RegistryEvidence) -> String {
    let mut lines = Vec::new();
    if let Some(publish_time) = &registry.publish_time {
        lines.push(format!("Published: {publish_time}"));
    }
    if !registry.maintainers.is_empty() {
        lines.push(format!(
            "Maintainers: {}",
            registry
                .maintainers
                .iter()
                .map(format_registry_person)
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if let Some(publisher) = &registry.publisher {
        lines.push(format!("Publisher: {}", format_registry_person(publisher)));
    }
    if let Some(repository) = &registry.repository {
        lines.push(format!("Repository: {}", redact_report_value(repository)));
    }
    if let Some(license) = &registry.license {
        lines.push(format!("License: {license}"));
    }
    if !registry.provenance.is_empty() {
        lines.push(format!(
            "Provenance: {}",
            registry
                .provenance
                .keys()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    render_optional_lines(&lines)
}

/// Render optional package metadata when present.
pub(crate) fn render_package_optional_evidence(evidence: &PackageOptionalEvidence) -> String {
    let mut lines = Vec::new();
    if let Some(repository) = &evidence.repository {
        lines.push(format!(
            "Package repository: {}",
            redact_report_value(repository)
        ));
    }
    if let Some(license) = &evidence.license {
        lines.push(format!("Package license: {license}"));
    }
    if !evidence.maintainers.is_empty() {
        lines.push(format!(
            "Package maintainers: {}",
            evidence
                .maintainers
                .iter()
                .map(format_package_person)
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if !evidence.provenance.is_empty() {
        lines.push(format!(
            "Package provenance: {}",
            evidence
                .provenance
                .keys()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    render_optional_lines(&lines)
}

/// Format a registry person without leaking raw secrets.
fn format_registry_person(person: &RegistryPerson) -> String {
    format_person(person.name.as_deref(), person.email.as_deref())
}

/// Format a package person without leaking raw secrets.
fn format_package_person(person: &PackagePerson) -> String {
    format_person(person.name.as_deref(), person.email.as_deref())
}

/// Format common npm person fields.
fn format_person(name: Option<&str>, email: Option<&str>) -> String {
    match (name, email) {
        (Some(name), Some(email)) => {
            format!(
                "{} <{}>",
                redact_report_value(name),
                redact_report_value(email)
            )
        }
        (Some(name), None) => redact_report_value(name),
        (None, Some(email)) => redact_report_value(email),
        (None, None) => "unknown".to_string(),
    }
}

/// Render optional evidence lines with a trailing newline only when non-empty.
fn render_optional_lines(lines: &[String]) -> String {
    if lines.is_empty() {
        return String::new();
    }

    format!("{}\n", lines.join("\n"))
}
