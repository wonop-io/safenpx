//! CLI behavior for the `safe-npx` execution gate.
//!
//! The scaffold is intentionally conservative: it reports the intended
//! inspection boundary without delegating to `npm exec` yet.

use clap::{Parser, ValueEnum};
use serde::Serialize;

/// Deterministic M2 package bin selection.
mod bin_selection;
/// M2 no-package-code-ran canary harness.
mod canary;
/// M2 execution closure evidence contracts.
mod closure;
/// M2 static closure blocker classification.
mod closure_blockers;
/// Shared M1 data contracts.
mod contracts;
/// M2 pinned delegation feasibility fixtures.
mod delegation_feasibility;
#[cfg(test)]
/// M2 direct-extract execution prototype for local fixture packages.
mod direct_execution;
/// Root tarball byte downloader.
mod download;
/// M2 executable byte identity for selected bins and generated shims.
mod executable_identity;
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
/// M2 execution-refusal report helpers.
mod m2_report;
/// M3 inspect-mode static extraction pipeline.
mod m3_inspect;
#[cfg(test)]
/// Tests for the M3 inspect-mode pipeline.
mod m3_inspect_tests;
/// Package metadata parsing helpers for static extraction.
mod package_metadata;
/// Package spec parser.
mod parser;
#[cfg(test)]
/// Reviewed process execution boundary.
mod process_boundary;
/// Deterministic M2 resolution-to-execution race fixtures.
mod race_matrix;
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

pub use bin_selection::*;
pub use canary::*;
pub use closure::*;
pub use closure_blockers::*;
pub use contracts::*;
pub use delegation_feasibility::*;
pub use download::*;
pub use executable_identity::*;
pub use extraction::*;
pub use fixtures::*;
pub use inspect::*;
pub use integrity::*;
pub use m2_fixtures::*;
pub use m3_inspect::*;
pub use parser::*;
pub use race_matrix::*;
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

    /// Emit a deterministic M2 refusal report for fixture and contract tests.
    #[arg(long, hide = true, value_enum)]
    pub m2_refusal: Option<M2RefusalReason>,

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
            .skip(self.action_token_count())
            .skip_while(|token| token.as_str() != "--")
            .skip(1)
            .cloned()
            .collect()
    }

    /// Return true when the caller selected the explicit M3 inspect action.
    pub fn is_inspect_action(&self) -> bool {
        self.command.first().map(String::as_str) == Some("inspect")
    }

    /// Return command tokens before the forwarded-args separator.
    fn spec_tokens(&self) -> Vec<String> {
        self.command
            .iter()
            .skip(self.action_token_count())
            .take_while(|token| token.as_str() != "--")
            .cloned()
            .collect()
    }

    /// Return the number of leading action tokens before the package spec.
    fn action_token_count(&self) -> usize {
        usize::from(self.is_inspect_action())
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

/// Hidden M2 refusal fixture reason accepted by the CLI contract path.
#[derive(Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum M2RefusalReason {
    /// Full dependency or execution closure cannot be proven.
    UnsupportedClosure,
    /// More than one package binary could match the command.
    AmbiguousBin,
    /// No package binary could be selected.
    MissingBin,
    /// Lifecycle script metadata blocks execution.
    LifecycleScriptPresent,
    /// Inspection and execution registry source would differ.
    RegistryPrecedenceMismatch,
    /// Prepared cache identity differs from inspected evidence.
    CacheIdentityMismatch,
    /// Generated shim identity differs from inspected evidence.
    ShimIdentityMismatch,
    /// Non-interactive mode must stop instead of prompting.
    NonInteractiveStop,
}

impl From<M2RefusalReason> for M2Reason {
    /// Convert a CLI fixture reason into the stable M2 reason vocabulary.
    fn from(reason: M2RefusalReason) -> Self {
        match reason {
            M2RefusalReason::UnsupportedClosure => Self::UnsupportedClosure,
            M2RefusalReason::AmbiguousBin => Self::AmbiguousBin,
            M2RefusalReason::MissingBin => Self::MissingBin,
            M2RefusalReason::LifecycleScriptPresent => Self::LifecycleScriptPresent,
            M2RefusalReason::RegistryPrecedenceMismatch => Self::RegistryPrecedenceMismatch,
            M2RefusalReason::CacheIdentityMismatch => Self::CacheIdentityMismatch,
            M2RefusalReason::ShimIdentityMismatch => Self::ShimIdentityMismatch,
            M2RefusalReason::NonInteractiveStop => Self::NonInteractiveStop,
        }
    }
}
