use clap::{Parser, ValueEnum};
use serde::Serialize;

#[derive(Debug, Parser)]
#[command(name = "safe-npx")]
#[command(about = "Evidence gate before npx/npm exec runs remote package code")]
struct Cli {
    /// Emit machine-readable JSON for agents and CI.
    #[arg(long)]
    json: bool,

    /// Print the demo output without executing anything.
    #[arg(long)]
    dry_run: bool,

    /// Decision to apply after inspection. v0.1 defaults to ask.
    #[arg(long, value_enum, default_value_t = Decision::Ask)]
    decision: Decision,

    /// Package spec, for example create-example@latest.
    package_spec: String,

    /// Arguments passed through to the package command after `--`.
    #[arg(last = true)]
    args: Vec<String>,
}

#[derive(Clone, Debug, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
enum Decision {
    Allow,
    Ask,
    Deny,
}

#[derive(Debug, Serialize)]
struct Report<'a> {
    package_spec: &'a str,
    recommendation: Decision,
    status: &'a str,
    note: &'a str,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let report = Report {
        package_spec: &cli.package_spec,
        recommendation: cli.decision.clone(),
        status: "scaffold",
        note: "Resolution, integrity verification, graph inspection, and execution delegation are not implemented yet.",
    };

    if cli.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    println!("Package: {}", cli.package_spec);
    println!("Status: scaffold");
    println!("Recommendation: {:?}", cli.decision);
    println!();
    println!("This Rust CLI scaffold does not execute package code yet.");
    println!("Next step: implement exact artifact resolution before delegation to npm exec.");

    Ok(())
}

