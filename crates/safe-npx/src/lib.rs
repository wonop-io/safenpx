//! CLI behavior for the `safe-npx` execution gate.
//!
//! The scaffold is intentionally conservative: it reports the intended
//! inspection boundary without delegating to `npm exec` yet.

use clap::{Parser, ValueEnum};
use serde::Serialize;

/// M2 no-package-code-ran canary harness.
mod canary;
/// M2 execution closure evidence contracts.
mod closure;
/// M2 static closure blocker classification.
mod closure_blockers;
/// Shared M1 data contracts.
mod contracts;
/// Root tarball byte downloader.
mod download;
/// Safe static extraction of verified root artifacts.
mod extraction;
#[cfg(test)]
/// Tests for safe static extraction.
mod extraction_tests;
/// M1 fixture manifest support.
mod fixtures;
/// Inspect boundary and no-network harness.
mod inspect;
/// Root artifact integrity verifier.
mod integrity;
/// M2 closure fixture manifest support.
mod m2_fixtures;
/// Package metadata parsing helpers for static extraction.
mod package_metadata;
/// Package spec parser.
mod parser;
/// Public npm registry metadata client.
mod registry;
/// Deterministic M2 registry precedence and agreement checks.
mod registry_precedence;
/// Human and JSON report rendering.
mod report;
#[cfg(test)]
/// Tests for M1 report rendering.
mod report_tests;
/// End-to-end root artifact resolver.
mod resolver;

pub use canary::*;
pub use closure::*;
pub use closure_blockers::*;
pub use contracts::*;
pub use download::*;
pub use extraction::*;
pub use fixtures::*;
pub use inspect::*;
pub use integrity::*;
pub use m2_fixtures::*;
pub use parser::*;
pub use registry::*;
pub use registry_precedence::*;
pub use report::*;
pub use resolver::*;

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
        if self.is_exec_variant_command() {
            return self.command.join(" ");
        }

        self.spec_tokens().join(" ")
    }

    /// Return arguments that should be passed through after classification.
    pub fn forwarded_args(&self) -> Vec<String> {
        if self.is_exec_variant_command() {
            return Vec::new();
        }

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

    /// Return true when the raw command is an unsupported npm/npx exec shape.
    fn is_exec_variant_command(&self) -> bool {
        matches!(
            self.command.first().map(String::as_str),
            Some("npm" | "npx" | "npm-exec")
        )
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
