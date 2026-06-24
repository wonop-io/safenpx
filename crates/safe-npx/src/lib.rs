//! CLI behavior for the `safe-npx` execution gate.
//!
//! The scaffold is intentionally conservative: it reports the intended
//! inspection boundary without delegating to `npm exec` yet.

use clap::{Parser, ValueEnum};
use serde::Serialize;

mod contracts;
mod parser;

pub use contracts::*;
pub use parser::*;

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

    /// Command tokens, split into package spec and forwarded args at `--`.
    #[arg(
        required = true,
        num_args = 1..,
        allow_hyphen_values = true,
        trailing_var_arg = true
    )]
    pub command: Vec<String>,
}

impl Cli {
    /// Return the raw package spec string that should be classified.
    pub fn raw_package_spec(&self) -> String {
        self.spec_tokens().join(" ")
    }

    /// Return arguments that should be passed through after classification.
    pub fn forwarded_args(&self) -> Vec<String> {
        self.command
            .iter()
            .skip_while(|token| token.as_str() != "--")
            .skip(1)
            .cloned()
            .collect()
    }

    /// Return command tokens before the forwarded-args separator.
    fn spec_tokens(&self) -> Vec<String> {
        self.command
            .iter()
            .take_while(|token| token.as_str() != "--")
            .cloned()
            .collect()
    }
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

/// Scaffold inspection report emitted for humans and agents.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Report {
    /// Package spec requested by the caller.
    pub package_spec: String,
    /// Parsed command intent before registry or artifact access.
    pub intent: CommandIntent,
    /// Current recommendation from the policy gate.
    pub recommendation: Decision,
    /// Implementation status for the scaffold.
    pub status: &'static str,
    /// Human-readable note explaining the current boundary.
    pub note: &'static str,
}

/// Build the current scaffold report from parsed CLI arguments.
pub fn build_report(cli: &Cli) -> Report {
    let raw_package_spec = cli.raw_package_spec();

    Report {
        package_spec: raw_package_spec.clone(),
        intent: parse_command_intent(&raw_package_spec, cli.forwarded_args()),
        recommendation: cli.decision.clone(),
        status: "scaffold",
        note: "Resolution, integrity verification, evidence extraction, and fail-closed execution proof are not implemented yet.",
    }
}

/// Render the report in the requested output format.
pub fn render_report(cli: &Cli, report: &Report) -> anyhow::Result<String> {
    if cli.json {
        return Ok(serde_json::to_string_pretty(report)?);
    }

    let intent_line = match &report.intent.package_spec {
        PackageSpecParse::Supported(package_spec) => format!(
            "Parsed: {}@{}\nForwarded args: {}\n",
            package_spec.name,
            package_spec.version,
            format_forwarded_args(&report.intent.forwarded_args)
        ),
        PackageSpecParse::Unsupported(unsupported) => format!(
            "Unsupported: {:?} ({:?})\nDownloaded: {}\n",
            unsupported.reason, unsupported.category, unsupported.downloaded
        ),
        PackageSpecParse::Malformed(malformed) => format!(
            "Malformed: {:?}\nDownloaded: {}\n",
            malformed.reason, malformed.downloaded
        ),
    };

    Ok(format!(
        "Package: {}\nStatus: scaffold\nRecommendation: {:?}\n{}\nThis Rust CLI scaffold does not execute package code yet.\nNext step: implement exact artifact resolution and fail closed when execution byte identity cannot be proven.\n",
        report.package_spec, cli.decision, intent_line
    ))
}

/// Run the CLI and return the output that should be written to stdout.
pub fn run(cli: &Cli) -> anyhow::Result<String> {
    let report = build_report(cli);
    render_report(cli, &report)
}

/// Format forwarded args for terminal output.
fn format_forwarded_args(args: &[String]) -> String {
    if args.is_empty() {
        return "[]".to_string();
    }

    args.join(" ")
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

        assert_eq!(cli.raw_package_spec(), "create-example@1.2.3");
        assert_eq!(cli.decision, Decision::Ask);
        assert!(!cli.json);
    }

    /// Verifies that parsed arguments become the scaffold report.
    #[test]
    fn builds_scaffold_report() {
        let cli = Cli::parse_from(["safe-npx", "--decision", "deny", "left-pad@1.3.0"]);
        let report = build_report(&cli);

        assert_eq!(report.package_spec, "left-pad@1.3.0");
        assert!(report.intent.is_supported());
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
        assert!(output.contains("\"state\": \"supported\""));
        assert!(output.contains("\"name\": \"create-example\""));
        assert!(output.contains("\"version\": \"1.2.3\""));
        assert!(output.contains("\"recommendation\": \"ask\""));
    }

    /// Verifies machine-readable output includes forwarded args.
    #[test]
    fn renders_json_with_forwarded_args_for_agents() {
        let cli = Cli::parse_from([
            "safe-npx",
            "--json",
            "create-example@1.2.3",
            "--",
            "--template",
            "react",
        ]);
        let output = run(&cli).expect("json rendering should succeed");

        assert!(output.contains("\"forwarded_args\": ["));
        assert!(output.contains("\"--template\""));
        assert!(output.contains("\"react\""));
    }

    /// Verifies unsupported specs are visible in public CLI JSON.
    #[test]
    fn renders_json_for_unsupported_specs() {
        let cli = Cli::parse_from(["safe-npx", "--json", "create-example@next"]);
        let output = run(&cli).expect("json rendering should succeed");

        assert!(output.contains("\"state\": \"unsupported\""));
        assert!(output.contains("\"reason\": \"unsupported_spec\""));
        assert!(output.contains("\"category\": \"version_range\""));
        assert!(output.contains("\"downloaded\": false"));
    }

    /// Verifies multi-token unsupported forms reach parser classification.
    #[test]
    fn renders_json_for_multi_token_unsupported_specs() {
        let cli = Cli::parse_from(["safe-npx", "--json", "npm", "exec", "create-example@1.2.3"]);
        let output = run(&cli).expect("json rendering should succeed");

        assert!(output.contains("\"package_spec\": \"npm exec create-example@1.2.3\""));
        assert!(output.contains("\"state\": \"unsupported\""));
        assert!(output.contains("\"category\": \"npm_exec_variant\""));
    }

    /// Verifies hyphenated npm/npx flags reach parser classification.
    #[test]
    fn renders_json_for_flagged_exec_variants() {
        let cases = [
            vec![
                "safe-npx",
                "--json",
                "npm",
                "exec",
                "--package",
                "create-example@1.2.3",
            ],
            vec![
                "safe-npx",
                "--json",
                "npm",
                "exec",
                "-c",
                "create-example@1.2.3 --help",
            ],
            vec!["safe-npx", "--json", "npx", "--yes", "create-example@1.2.3"],
        ];

        for case in cases {
            let cli = Cli::parse_from(case);
            let output = run(&cli).expect("json rendering should succeed");

            assert!(output.contains("\"state\": \"unsupported\""));
            assert!(output.contains("\"category\": \"npm_exec_variant\""));
            assert!(output.contains("\"downloaded\": false"));
        }
    }

    /// Verifies human-readable scaffold output for terminal use.
    #[test]
    fn renders_human_scaffold_output() {
        let cli = Cli::parse_from(["safe-npx", "--decision", "allow", "create-example@1.2.3"]);
        let output = run(&cli).expect("text rendering should succeed");

        assert!(output.contains("Package: create-example@1.2.3"));
        assert!(output.contains("Parsed: create-example@1.2.3"));
        assert!(output.contains("Recommendation: Allow"));
        assert!(output.contains("does not execute package code yet"));
    }
}
