//! CLI behavior for the `safe-npx` execution gate.
//!
//! The scaffold is intentionally conservative: it reports the intended
//! inspection boundary without delegating to `npm exec` yet.

use clap::{Parser, ValueEnum};
use serde::Serialize;

/// Command-line arguments accepted by the `safe-npx` binary.
#[derive(Debug, Parser)]
#[command(name = "safe-npx")]
#[command(about = "Evidence gate before npx/npm exec runs remote package code")]
pub struct Cli {
    /// Emit machine-readable JSON for agents and CI.
    #[arg(long)]
    pub json: bool,

    /// Print the demo output without executing anything.
    #[arg(long)]
    pub dry_run: bool,

    /// Decision to apply after inspection. v0.1 defaults to ask.
    #[arg(long, value_enum, default_value_t = Decision::Ask)]
    pub decision: Decision,

    /// Package spec, for example create-example@1.2.3.
    pub package_spec: String,

    /// Arguments passed through to the package command after `--`.
    #[arg(last = true)]
    pub args: Vec<String>,
}

/// Policy decision selected before package execution.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    /// Allow execution after inspection.
    Allow,
    /// Ask the user before execution.
    Ask,
    /// Deny execution after inspection.
    Deny,
}

/// Caller intent after CLI parsing, before any registry or artifact access.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CommandIntent {
    /// Raw package spec requested by the caller.
    pub requested: String,
    /// Parsed package spec state.
    pub package_spec: PackageSpecParse,
    /// Arguments that should be passed to the selected package binary.
    pub forwarded_args: Vec<String>,
}

impl CommandIntent {
    /// Build an intent for a supported exact-version package spec.
    pub fn supported(package_spec: PackageSpec, forwarded_args: Vec<String>) -> Self {
        Self {
            requested: package_spec.raw.clone(),
            package_spec: PackageSpecParse::Supported(package_spec),
            forwarded_args,
        }
    }

    /// Build an intent for a spec that must not reach network or execution code.
    pub fn unsupported(requested: impl Into<String>, unsupported: UnsupportedSpec) -> Self {
        Self {
            requested: requested.into(),
            package_spec: PackageSpecParse::Unsupported(unsupported),
            forwarded_args: Vec::new(),
        }
    }

    /// Return true when this intent can move into resolver work.
    pub fn is_supported(&self) -> bool {
        matches!(self.package_spec, PackageSpecParse::Supported(_))
    }
}

/// Parsed package spec state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum PackageSpecParse {
    /// Spec is inside the M1 supported exact-version surface.
    Supported(PackageSpec),
    /// Spec is recognized but intentionally outside the supported surface.
    Unsupported(UnsupportedSpec),
    /// Spec cannot be parsed well enough to classify safely.
    Malformed(MalformedSpec),
}

/// Supported npm package spec pinned to an exact version.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PackageSpec {
    /// Original spec string as supplied by the caller.
    pub raw: String,
    /// Package name, including scope when present.
    pub name: String,
    /// Exact version requested by the caller.
    pub version: String,
    /// Optional npm scope without the leading `@`.
    pub scope: Option<String>,
}

impl PackageSpec {
    /// Create an exact-version package spec contract.
    pub fn exact(
        raw: impl Into<String>,
        name: impl Into<String>,
        version: impl Into<String>,
        scope: Option<String>,
    ) -> Self {
        Self {
            raw: raw.into(),
            name: name.into(),
            version: version.into(),
            scope,
        }
    }
}

/// Unsupported spec classification that should fail closed before network work.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct UnsupportedSpec {
    /// Stable M1 reason for the refusal.
    pub reason: M1Reason,
    /// Human-readable category of unsupported input.
    pub category: UnsupportedSpecCategory,
    /// Whether any registry or tarball bytes were downloaded before refusal.
    pub downloaded: bool,
}

/// Unsupported package-spec categories named by the M1 matrix.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnsupportedSpecCategory {
    /// Package name without a version.
    UnversionedName,
    /// Version range or dist-tag outside the current supported path.
    VersionRange,
    /// Git dependency URL or shorthand.
    GitUrl,
    /// Local filesystem package path.
    LocalPath,
    /// Direct tarball URL.
    TarballUrl,
    /// npm package alias.
    Alias,
    /// More than one package spec was supplied.
    MultipleSpecs,
    /// Unsupported `npm exec` command shape.
    NpmExecVariant,
    /// Other known unsupported shape.
    Other,
}

/// Malformed spec contract for inputs that cannot safely proceed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct MalformedSpec {
    /// Stable M1 reason for the refusal.
    pub reason: M1Reason,
    /// Original malformed input.
    pub raw: String,
    /// Whether any registry or tarball bytes were downloaded before refusal.
    pub downloaded: bool,
}

/// Registry source used to resolve npm metadata.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RegistrySource {
    /// Registry URL used for metadata and tarball lookup.
    pub url: String,
    /// Optional scope that selected this registry.
    pub scope: Option<String>,
}

/// Exact package coordinates resolved from registry metadata.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ResolvedPackage {
    /// Package name, including scope when present.
    pub name: String,
    /// Exact package version.
    pub version: String,
    /// Registry used to resolve the package.
    pub registry: RegistrySource,
    /// Tarball URL from registry metadata.
    pub tarball_url: String,
    /// Integrity string from registry metadata.
    pub integrity: String,
}

/// Stable identity for the downloaded root artifact bytes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ArtifactIdentity {
    /// Package name, including scope when present.
    pub name: String,
    /// Exact package version.
    pub version: String,
    /// Integrity metadata verified for the artifact.
    pub integrity: String,
    /// Digest algorithm used by `digest`.
    pub digest_algorithm: String,
    /// Digest of the exact artifact bytes.
    pub digest: String,
}

/// Stable M1 reason vocabulary for resolver and artifact outcomes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum M1Reason {
    /// The package spec is outside the current supported surface.
    UnsupportedSpec,
    /// The package spec is malformed.
    MalformedSpec,
    /// Registry metadata could not be fetched or interpreted.
    RegistryError,
    /// Downloaded bytes did not match registry integrity metadata.
    IntegrityMismatch,
    /// Package name does not exist in the selected registry.
    MissingPackage,
    /// Exact version does not exist for the package.
    MissingVersion,
}

/// Scaffold inspection report emitted for humans and agents.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Report<'a> {
    /// Package spec requested by the caller.
    pub package_spec: &'a str,
    /// Current recommendation from the policy gate.
    pub recommendation: Decision,
    /// Implementation status for the scaffold.
    pub status: &'a str,
    /// Human-readable note explaining the current boundary.
    pub note: &'a str,
}

/// Build the current scaffold report from parsed CLI arguments.
pub fn build_report(cli: &Cli) -> Report<'_> {
    Report {
        package_spec: &cli.package_spec,
        recommendation: cli.decision.clone(),
        status: "scaffold",
        note: "Resolution, integrity verification, evidence extraction, and fail-closed execution proof are not implemented yet.",
    }
}

/// Render the report in the requested output format.
pub fn render_report(cli: &Cli, report: &Report<'_>) -> anyhow::Result<String> {
    if cli.json {
        return Ok(serde_json::to_string_pretty(report)?);
    }

    Ok(format!(
        "Package: {}\nStatus: scaffold\nRecommendation: {:?}\n\nThis Rust CLI scaffold does not execute package code yet.\nNext step: implement exact artifact resolution and fail closed when execution byte identity cannot be proven.\n",
        cli.package_spec, cli.decision
    ))
}

/// Run the CLI and return the output that should be written to stdout.
pub fn run(cli: &Cli) -> anyhow::Result<String> {
    let report = build_report(cli);
    render_report(cli, &report)
}

/// Unit tests for CLI parsing and scaffold rendering.
#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    /// Verifies that the CLI defaults to asking before execution.
    #[test]
    fn parses_default_ask_decision() {
        let cli = Cli::parse_from(["safe-npx", "create-example@1.2.3"]);

        assert_eq!(cli.package_spec, "create-example@1.2.3");
        assert_eq!(cli.decision, Decision::Ask);
        assert!(!cli.json);
    }

    /// Verifies that parsed arguments become the scaffold report.
    #[test]
    fn builds_scaffold_report() {
        let cli = Cli::parse_from(["safe-npx", "--decision", "deny", "left-pad@1.3.0"]);
        let report = build_report(&cli);

        assert_eq!(report.package_spec, "left-pad@1.3.0");
        assert_eq!(report.recommendation, Decision::Deny);
        assert_eq!(report.status, "scaffold");
        assert!(report.note.contains("not implemented yet"));
    }

    /// Verifies machine-readable output for agent workflows.
    #[test]
    fn renders_json_for_agents() {
        let cli = Cli::parse_from(["safe-npx", "--json", "create-example@1.2.3"]);
        let output = run(&cli).expect("json rendering should succeed");

        assert!(output.contains("\"package_spec\": \"create-example@1.2.3\""));
        assert!(output.contains("\"recommendation\": \"ask\""));
    }

    /// Verifies human-readable scaffold output for terminal use.
    #[test]
    fn renders_human_scaffold_output() {
        let cli = Cli::parse_from(["safe-npx", "--decision", "allow", "create-example@1.2.3"]);
        let output = run(&cli).expect("text rendering should succeed");

        assert!(output.contains("Package: create-example@1.2.3"));
        assert!(output.contains("Recommendation: Allow"));
        assert!(output.contains("does not execute package code yet"));
    }

    /// Verifies that exact-version specs and forwarded args have a stable shape.
    #[test]
    fn command_intent_represents_supported_exact_version_specs() {
        let spec = PackageSpec::exact(
            "@scope/create-example@1.2.3",
            "@scope/create-example",
            "1.2.3",
            Some("scope".to_string()),
        );
        let intent =
            CommandIntent::supported(spec, vec!["--template".to_string(), "react".to_string()]);

        assert!(intent.is_supported());
        assert_eq!(intent.requested, "@scope/create-example@1.2.3");
        assert_eq!(intent.forwarded_args, ["--template", "react"]);
        let json = serde_json::to_string(&intent).expect("intent should serialize");
        assert!(json.contains("\"state\":\"supported\""));
        assert!(json.contains("\"scope\":\"scope\""));
    }

    /// Verifies unsupported specs are representable before network work.
    #[test]
    fn command_intent_represents_unsupported_specs_without_downloads() {
        let intent = CommandIntent::unsupported(
            "create-example",
            UnsupportedSpec {
                reason: M1Reason::UnsupportedSpec,
                category: UnsupportedSpecCategory::UnversionedName,
                downloaded: false,
            },
        );

        assert!(!intent.is_supported());
        assert!(intent.forwarded_args.is_empty());
        let json = serde_json::to_string(&intent).expect("intent should serialize");
        assert!(json.contains("\"reason\":\"unsupported_spec\""));
        assert!(json.contains("\"category\":\"unversioned_name\""));
        assert!(json.contains("\"downloaded\":false"));
    }

    /// Verifies malformed specs and the minimum reason vocabulary serialize.
    #[test]
    fn malformed_specs_and_m1_reasons_are_serializable() {
        let malformed = PackageSpecParse::Malformed(MalformedSpec {
            reason: M1Reason::MalformedSpec,
            raw: "@scope/".to_string(),
            downloaded: false,
        });
        let reasons = [
            M1Reason::UnsupportedSpec,
            M1Reason::MalformedSpec,
            M1Reason::RegistryError,
            M1Reason::IntegrityMismatch,
        ];

        let json = serde_json::to_string(&malformed).expect("malformed spec should serialize");
        assert!(json.contains("\"reason\":\"malformed_spec\""));
        assert!(json.contains("\"downloaded\":false"));
        assert_eq!(
            serde_json::to_value(reasons).expect("reasons should serialize"),
            serde_json::json!([
                "unsupported_spec",
                "malformed_spec",
                "registry_error",
                "integrity_mismatch"
            ])
        );
    }

    /// Verifies resolved package and artifact identity contracts compose cleanly.
    #[test]
    fn resolved_package_and_artifact_identity_are_stable_contracts() {
        let registry = RegistrySource {
            url: "https://registry.npmjs.org".to_string(),
            scope: None,
        };
        let resolved = ResolvedPackage {
            name: "create-example".to_string(),
            version: "1.2.3".to_string(),
            registry,
            tarball_url: "https://registry.npmjs.org/create-example/-/create-example-1.2.3.tgz"
                .to_string(),
            integrity: "sha512-example".to_string(),
        };
        let identity = ArtifactIdentity {
            name: resolved.name.clone(),
            version: resolved.version.clone(),
            integrity: resolved.integrity.clone(),
            digest_algorithm: "sha512".to_string(),
            digest: "example".to_string(),
        };

        assert_eq!(identity.name, "create-example");
        assert_eq!(identity.version, "1.2.3");
        assert_eq!(identity.integrity, "sha512-example");
        assert_eq!(identity.digest_algorithm, "sha512");
    }
}
