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

    /// Package spec, for example create-example@latest.
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
        note: "Resolution, integrity verification, graph inspection, and execution delegation are not implemented yet.",
    }
}

/// Render the report in the requested output format.
pub fn render_report(cli: &Cli, report: &Report<'_>) -> anyhow::Result<String> {
    if cli.json {
        return Ok(serde_json::to_string_pretty(report)?);
    }

    Ok(format!(
        "Package: {}\nStatus: scaffold\nRecommendation: {:?}\n\nThis Rust CLI scaffold does not execute package code yet.\nNext step: implement exact artifact resolution before delegation to npm exec.\n",
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
        let cli = Cli::parse_from(["safe-npx", "create-example@latest"]);

        assert_eq!(cli.package_spec, "create-example@latest");
        assert_eq!(cli.decision, Decision::Ask);
        assert!(!cli.json);
    }

    /// Verifies that parsed arguments become the scaffold report.
    #[test]
    fn builds_scaffold_report() {
        let cli = Cli::parse_from(["safe-npx", "--decision", "deny", "left-pad@latest"]);
        let report = build_report(&cli);

        assert_eq!(report.package_spec, "left-pad@latest");
        assert_eq!(report.recommendation, Decision::Deny);
        assert_eq!(report.status, "scaffold");
        assert!(report.note.contains("not implemented yet"));
    }

    /// Verifies machine-readable output for agent workflows.
    #[test]
    fn renders_json_for_agents() {
        let cli = Cli::parse_from(["safe-npx", "--json", "create-example@latest"]);
        let output = run(&cli).expect("json rendering should succeed");

        assert!(output.contains("\"package_spec\": \"create-example@latest\""));
        assert!(output.contains("\"recommendation\": \"ask\""));
    }

    /// Verifies human-readable scaffold output for terminal use.
    #[test]
    fn renders_human_scaffold_output() {
        let cli = Cli::parse_from(["safe-npx", "--decision", "allow", "create-example@latest"]);
        let output = run(&cli).expect("text rendering should succeed");

        assert!(output.contains("Package: create-example@latest"));
        assert!(output.contains("Recommendation: Allow"));
        assert!(output.contains("does not execute package code yet"));
    }
}
