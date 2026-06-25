//! M2 static closure blocker classification.

use crate::{
    ClosureDecision, DependencyDeclaration, ExtractedPackageMetadata, LifecycleScript, M2Reason,
};

/// Static blocker assessment for an execute candidate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClosureBlockerAssessment {
    /// Decision implied by static metadata blockers.
    pub decision: ClosureDecision,
    /// Stable reasons supporting the decision.
    pub reasons: Vec<M2Reason>,
    /// Lifecycle scripts detected in root metadata.
    pub lifecycle_scripts: Vec<LifecycleScript>,
    /// Dependency declarations recorded from root metadata.
    pub dependency_declarations: Vec<DependencyDeclaration>,
    /// Dependency declarations that block M2 execution.
    pub blocking_dependency_declarations: Vec<DependencyDeclaration>,
}

impl ClosureBlockerAssessment {
    /// Return true when static metadata contains no M2 execution blockers.
    pub fn is_clear(&self) -> bool {
        self.decision == ClosureDecision::Ask
            && self.reasons == vec![M2Reason::InteractiveApprovalRequired]
            && self.lifecycle_scripts.is_empty()
            && self.blocking_dependency_declarations.is_empty()
    }
}

/// Assess lifecycle scripts and dependency declarations as M2 closure blockers.
pub fn assess_static_closure_blockers(
    metadata: &ExtractedPackageMetadata,
) -> ClosureBlockerAssessment {
    let lifecycle_scripts = metadata
        .lifecycle_scripts
        .iter()
        .map(|(name, command)| LifecycleScript {
            name: name.clone(),
            command: command.clone(),
        })
        .collect::<Vec<_>>();
    let dependency_declarations = metadata
        .dependency_declarations
        .iter()
        .map(|declaration| DependencyDeclaration {
            name: declaration.name.clone(),
            requirement: declaration.requirement.clone(),
            kind: declaration.kind.clone(),
        })
        .collect::<Vec<_>>();
    let blocking_dependency_declarations = dependency_declarations
        .iter()
        .filter(|declaration| declaration.kind.requires_m2_dependency_closure())
        .cloned()
        .collect::<Vec<_>>();

    let mut reasons = Vec::new();
    if !lifecycle_scripts.is_empty() {
        reasons.push(M2Reason::LifecycleScriptPresent);
    }
    if !blocking_dependency_declarations.is_empty() {
        reasons.push(M2Reason::UnsupportedClosure);
    }
    if reasons.is_empty() {
        reasons.push(M2Reason::InteractiveApprovalRequired);
    }

    let decision = if lifecycle_scripts.is_empty() && blocking_dependency_declarations.is_empty() {
        ClosureDecision::Ask
    } else {
        ClosureDecision::ExecutionRefused
    };

    ClosureBlockerAssessment {
        decision,
        reasons,
        lifecycle_scripts,
        dependency_declarations,
        blocking_dependency_declarations,
    }
}

#[cfg(test)]
/// Tests for static closure blocker classification.
mod tests {
    use super::*;
    use crate::{
        DependencyDeclarationKind, ExtractedDependencyDeclaration, ExtractedPackageMetadata,
    };
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    #[test]
    /// Verifies packages without scripts or install-time deps remain promptable.
    fn no_deps_no_scripts_remains_interactive() {
        let assessment = assess_static_closure_blockers(&metadata(BTreeMap::new(), vec![]));

        assert!(assessment.is_clear());
        assert_eq!(assessment.decision, ClosureDecision::Ask);
        assert_eq!(
            assessment.reasons,
            vec![M2Reason::InteractiveApprovalRequired]
        );
    }

    #[test]
    /// Verifies lifecycle scripts refuse execution before package code can run.
    fn lifecycle_scripts_refuse_execution() {
        let assessment = assess_static_closure_blockers(&metadata(
            BTreeMap::from([("postinstall".to_string(), "node postinstall.js".to_string())]),
            vec![],
        ));

        assert_eq!(assessment.decision, ClosureDecision::ExecutionRefused);
        assert_eq!(assessment.reasons, vec![M2Reason::LifecycleScriptPresent]);
        assert_eq!(assessment.lifecycle_scripts[0].name, "postinstall");
    }

    #[test]
    /// Verifies runtime dependencies refuse execution until closure proof exists.
    fn runtime_dependencies_refuse_execution() {
        let assessment = assess_static_closure_blockers(&metadata(
            BTreeMap::new(),
            vec![dependency(
                "left-pad",
                "^1.3.0",
                DependencyDeclarationKind::Runtime,
            )],
        ));

        assert_eq!(assessment.decision, ClosureDecision::ExecutionRefused);
        assert_eq!(assessment.reasons, vec![M2Reason::UnsupportedClosure]);
        assert_eq!(assessment.blocking_dependency_declarations.len(), 1);
    }

    #[test]
    /// Verifies optional dependencies are explicitly classified as blockers.
    fn optional_dependencies_refuse_execution() {
        let assessment = assess_static_closure_blockers(&metadata(
            BTreeMap::new(),
            vec![dependency(
                "fsevents",
                "^2.3.0",
                DependencyDeclarationKind::Optional,
            )],
        ));

        assert_eq!(assessment.decision, ClosureDecision::ExecutionRefused);
        assert_eq!(
            assessment.blocking_dependency_declarations[0].kind,
            DependencyDeclarationKind::Optional
        );
    }

    #[test]
    /// Verifies peer dependencies are explicitly classified as blockers.
    fn peer_dependencies_refuse_execution() {
        let assessment = assess_static_closure_blockers(&metadata(
            BTreeMap::new(),
            vec![dependency("react", "^18", DependencyDeclarationKind::Peer)],
        ));

        assert_eq!(assessment.decision, ClosureDecision::ExecutionRefused);
        assert_eq!(
            assessment.blocking_dependency_declarations[0].kind,
            DependencyDeclarationKind::Peer
        );
    }

    #[test]
    /// Verifies bundled dependency declarations are not assumed safe.
    fn bundled_dependencies_refuse_execution() {
        let assessment = assess_static_closure_blockers(&metadata(
            BTreeMap::new(),
            vec![dependency(
                "bundled-tool",
                "bundled",
                DependencyDeclarationKind::Bundled,
            )],
        ));

        assert_eq!(assessment.decision, ClosureDecision::ExecutionRefused);
        assert_eq!(
            assessment.blocking_dependency_declarations[0].kind,
            DependencyDeclarationKind::Bundled
        );
    }

    #[test]
    /// Verifies dev dependencies and peer metadata are recorded but not blockers.
    fn non_runtime_metadata_does_not_block_m2_candidate() {
        let assessment = assess_static_closure_blockers(&metadata(
            BTreeMap::new(),
            vec![
                dependency("ava", "^6", DependencyDeclarationKind::Development),
                dependency(
                    "react",
                    r#"{"optional":true}"#,
                    DependencyDeclarationKind::PeerMetadata,
                ),
            ],
        ));

        assert!(assessment.is_clear());
        assert_eq!(assessment.dependency_declarations.len(), 2);
        assert!(assessment.blocking_dependency_declarations.is_empty());
    }

    #[test]
    /// Verifies lifecycle and dependency blockers can both be reported.
    fn lifecycle_and_dependency_reasons_are_both_reported() {
        let assessment = assess_static_closure_blockers(&metadata(
            BTreeMap::from([("prepare".to_string(), "node build.js".to_string())]),
            vec![dependency(
                "left-pad",
                "^1.3.0",
                DependencyDeclarationKind::Runtime,
            )],
        ));

        assert_eq!(assessment.decision, ClosureDecision::ExecutionRefused);
        assert_eq!(
            assessment.reasons,
            vec![
                M2Reason::LifecycleScriptPresent,
                M2Reason::UnsupportedClosure
            ]
        );
    }

    fn metadata(
        lifecycle_scripts: BTreeMap<String, String>,
        dependency_declarations: Vec<ExtractedDependencyDeclaration>,
    ) -> ExtractedPackageMetadata {
        ExtractedPackageMetadata {
            name: Some("create-example".to_string()),
            version: Some("1.2.3".to_string()),
            bins: BTreeMap::new(),
            lifecycle_scripts,
            dependency_declarations,
            package_json_path: PathBuf::from("package/package.json"),
        }
    }

    fn dependency(
        name: &str,
        requirement: &str,
        kind: DependencyDeclarationKind,
    ) -> ExtractedDependencyDeclaration {
        ExtractedDependencyDeclaration {
            name: name.to_string(),
            requirement: requirement.to_string(),
            kind,
        }
    }
}
