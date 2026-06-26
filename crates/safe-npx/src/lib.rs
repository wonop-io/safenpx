//! CLI behavior for the `safe-npx` execution gate.
//!
//! The scaffold is intentionally conservative: it reports the intended
//! inspection boundary without delegating to `npm exec` yet.

use clap::{Parser, ValueEnum};
use serde::Serialize;
use std::ffi::OsString;

/// Authority-context classification and redaction.
mod authority_context;
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
/// Shared M3 inspect evidence model.
mod inspect_model;
#[cfg(test)]
/// Tests for the shared M3 inspect evidence model.
mod inspect_model_tests;
/// Root artifact integrity verifier.
mod integrity;
/// M2 closure fixture manifest support.
mod m2_fixtures;
/// M2 execution-refusal report helpers.
mod m2_report;
/// M3 inspect-mode static extraction pipeline.
mod m3_inspect;
#[cfg(test)]
/// Canary carryover tests for the M3 inspect-mode pipeline.
mod m3_inspect_canary_tests;
#[cfg(test)]
/// Tests for the M3 inspect-mode pipeline.
mod m3_inspect_tests;
/// Optional package evidence parsed from verified package metadata.
mod package_evidence;
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
/// Registry metadata evidence extraction.
mod registry_evidence;
/// Deterministic M2 registry precedence and agreement checks.
mod registry_precedence;
/// Human and JSON report rendering.
mod report;
/// Shared inspect-model report helpers.
mod report_inspect;
#[cfg(test)]
/// Tests for M1 report rendering.
mod report_tests;
/// End-to-end root artifact resolver.
mod resolver;

pub use authority_context::*;
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
pub use inspect_model::*;
pub use integrity::*;
pub use m2_fixtures::*;
pub use m3_inspect::*;
pub use package_evidence::*;
pub use parser::*;
pub use race_matrix::*;
pub use registry::*;
pub use registry_evidence::*;
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

    /// Caller-declared source context. Defaults to unknown; safe-npx does not infer intent.
    #[arg(long, value_enum, default_value_t = SourceContext::Unknown)]
    pub source_context: SourceContext,

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
    /// Parse process arguments after normalizing inspect-local source context.
    pub fn parse() -> Self {
        Self::parse_from(std::env::args_os())
    }

    /// Parse provided arguments after normalizing inspect-local source context.
    pub fn parse_from<I, T>(args: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString>,
    {
        match Self::try_parse_from(args) {
            Ok(cli) => cli,
            Err(error) => error.exit(),
        }
    }

    /// Try to parse provided arguments after normalizing inspect-local source context.
    pub fn try_parse_from<I, T>(args: I) -> Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString>,
    {
        let normalized_args = normalize_inspect_source_context(args);
        <Self as Parser>::try_parse_from(normalized_args)
    }

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

/// Normalize `safe-npx inspect --source-context VALUE pkg` before Clap parsing.
fn normalize_inspect_source_context<I, T>(args: I) -> Vec<OsString>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString>,
{
    let mut args = args.into_iter().map(Into::into).collect::<Vec<_>>();
    let Some(inspect_index) = first_command_index(&args) else {
        return args;
    };
    if args.get(inspect_index).and_then(|arg| arg.to_str()) != Some("inspect") {
        return args;
    }

    let mut cursor = inspect_index + 1;
    while cursor < args.len() {
        let Some(token) = args.get(cursor).and_then(|arg| arg.to_str()) else {
            cursor += 1;
            continue;
        };
        if token == "--" {
            break;
        }
        if let Some(value) = token.strip_prefix("--source-context=") {
            let value = OsString::from(value);
            args.remove(cursor);
            args.insert(inspect_index, value);
            args.insert(inspect_index, OsString::from("--source-context"));
            break;
        }
        if token == "--source-context" {
            let value = if cursor + 1 < args.len() {
                args.remove(cursor + 1)
            } else {
                OsString::new()
            };
            args.remove(cursor);
            args.insert(inspect_index, value);
            args.insert(inspect_index, OsString::from("--source-context"));
            break;
        }
        cursor += 1;
    }

    args
}

/// Return the first command token after supported top-level options.
fn first_command_index(args: &[OsString]) -> Option<usize> {
    let mut index = 1;
    while index < args.len() {
        let token = args[index].to_str()?;
        if token == "--" {
            return None;
        }
        if token == "--json" || token == "--dry-run" {
            index += 1;
            continue;
        }
        if token == "--decision" || token == "--source-context" || token == "--m2-refusal" {
            index += 2;
            continue;
        }
        if token.starts_with("--decision=")
            || token.starts_with("--source-context=")
            || token.starts_with("--m2-refusal=")
        {
            index += 1;
            continue;
        }
        return Some(index);
    }

    None
}

/// Caller-declared source context for an inspect request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum SourceContext {
    /// Human-declared manual terminal invocation.
    #[value(name = "manual_terminal")]
    ManualTerminal,
    /// Command copied or adapted from README, docs, blog, or tutorial text.
    #[value(name = "docs_snippet")]
    DocsSnippet,
    /// Coding agent, skill, or playbook declared the request.
    #[value(name = "agent_skill")]
    AgentSkill,
    /// CI workflow, job, or automation declared the request.
    #[value(name = "ci")]
    Ci,
    /// No trusted source-context declaration was provided.
    #[value(name = "unknown")]
    Unknown,
}

impl Default for SourceContext {
    fn default() -> Self {
        Self::Unknown
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
