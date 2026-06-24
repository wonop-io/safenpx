//! Inspect boundary that proves parser failures stop before network hooks.

use crate::{parse_command_intent, CommandIntent, PackageSpec, PackageSpecParse};

/// Malformed package specs that must not reach network-capable hooks.
pub const MALFORMED_NO_NETWORK_CASES: &[&str] = &["", "@scope/", "create-example@", "@/pkg@1.2.3"];

/// Unsupported package specs that must not reach network-capable hooks.
pub const UNSUPPORTED_NO_NETWORK_CASES: &[&str] = &[
    "create-example",
    "create-example@latest",
    "create-example@^1.2.3",
    "@scope/create-example@latest",
    "@scope/create-example@^1.2.3",
    "github:user/repo",
    "./local-package",
    "C:tmp@1.2.3",
    "https://example.test/pkg.tgz",
    "alias@npm:create-example@1.2.3",
    "npm exec -- create-example@1.2.3",
    "npm exec --package create-example@1.2.3",
    "npm exec -c create-example@1.2.3",
    "npx --yes create-example@1.2.3",
    "npm-exec create-example@1.2.3",
    "create-example@1.2.3 other@2.0.0",
];

/// Network-capable hooks used after parser classification.
pub trait NetworkProbe {
    /// Record or perform a registry metadata lookup for a supported spec.
    fn resolve_metadata(&mut self, package_spec: &PackageSpec);

    /// Record or perform a tarball download for a supported spec.
    fn download_tarball(&mut self, package_spec: &PackageSpec);
}

/// Counter probe for reusable no-network tests.
#[derive(Default)]
pub struct CountingProbe {
    /// Number of registry metadata attempts.
    pub registry_calls: usize,
    /// Number of tarball download attempts.
    pub tarball_calls: usize,
}

impl NetworkProbe for CountingProbe {
    fn resolve_metadata(&mut self, _package_spec: &PackageSpec) {
        self.registry_calls += 1;
    }

    fn download_tarball(&mut self, _package_spec: &PackageSpec) {
        self.tarball_calls += 1;
    }
}

/// Inspect parsed intent and only enter network-capable hooks for supported specs.
pub fn inspect_with_probe(intent: &CommandIntent, probe: &mut impl NetworkProbe) {
    if let PackageSpecParse::Supported(package_spec) = &intent.package_spec {
        probe.resolve_metadata(package_spec);
        probe.download_tarball(package_spec);
    }
}

/// Parse a raw spec and inspect it through the network-capable boundary.
pub fn inspect_raw_spec_with_probe(
    raw_spec: &str,
    forwarded_args: Vec<String>,
    probe: &mut impl NetworkProbe,
) -> CommandIntent {
    let intent = parse_command_intent(raw_spec, forwarded_args);
    inspect_with_probe(&intent, probe);
    intent
}

/// Assert that a raw spec does not reach network-capable hooks.
pub fn assert_no_network_for_raw_spec(raw_spec: &str) {
    let mut probe = CountingProbe::default();
    let intent = inspect_raw_spec_with_probe(raw_spec, Vec::new(), &mut probe);

    assert!(
        !matches!(intent.package_spec, PackageSpecParse::Supported(_)),
        "{raw_spec}"
    );
    assert_eq!(probe.registry_calls, 0, "{raw_spec}");
    assert_eq!(probe.tarball_calls, 0, "{raw_spec}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_specs_do_not_reach_network_hooks() {
        for raw in MALFORMED_NO_NETWORK_CASES {
            assert_no_network_for_raw_spec(raw);
        }
    }

    #[test]
    fn unsupported_specs_do_not_reach_network_hooks() {
        for raw in UNSUPPORTED_NO_NETWORK_CASES {
            assert_no_network_for_raw_spec(raw);
        }
    }

    #[test]
    fn fixture_manifest_matches_no_network_cases() {
        let manifest = include_str!("../fixtures/no-network-manifest.txt");
        let expected = expected_manifest_lines();
        let actual = manifest.lines().collect::<Vec<_>>();

        assert_eq!(actual, expected);
    }

    #[test]
    fn supported_specs_enter_network_hooks() {
        let mut probe = CountingProbe::default();

        inspect_raw_spec_with_probe("create-example@1.2.3", Vec::new(), &mut probe);

        assert_eq!(probe.registry_calls, 1);
        assert_eq!(probe.tarball_calls, 1);
    }

    fn expected_manifest_lines() -> Vec<&'static str> {
        let mut lines = vec![
            "# safe-npx no-network fixture manifest",
            "# format: kind|raw_spec|registry_calls|tarball_calls",
        ];
        lines.extend(
            MALFORMED_NO_NETWORK_CASES
                .iter()
                .map(|raw| manifest_line("malformed", raw)),
        );
        lines.extend(
            UNSUPPORTED_NO_NETWORK_CASES
                .iter()
                .map(|raw| manifest_line("unsupported", raw)),
        );
        lines
    }

    fn manifest_line(kind: &str, raw: &str) -> &'static str {
        match (kind, raw) {
            ("malformed", "") => "malformed||0|0",
            ("malformed", "@scope/") => "malformed|@scope/|0|0",
            ("malformed", "create-example@") => "malformed|create-example@|0|0",
            ("malformed", "@/pkg@1.2.3") => "malformed|@/pkg@1.2.3|0|0",
            ("unsupported", "create-example") => "unsupported|create-example|0|0",
            ("unsupported", "create-example@latest") => "unsupported|create-example@latest|0|0",
            ("unsupported", "create-example@^1.2.3") => "unsupported|create-example@^1.2.3|0|0",
            ("unsupported", "@scope/create-example@latest") => {
                "unsupported|@scope/create-example@latest|0|0"
            }
            ("unsupported", "@scope/create-example@^1.2.3") => {
                "unsupported|@scope/create-example@^1.2.3|0|0"
            }
            ("unsupported", "github:user/repo") => "unsupported|github:user/repo|0|0",
            ("unsupported", "./local-package") => "unsupported|./local-package|0|0",
            ("unsupported", "C:tmp@1.2.3") => "unsupported|C:tmp@1.2.3|0|0",
            ("unsupported", "https://example.test/pkg.tgz") => {
                "unsupported|https://example.test/pkg.tgz|0|0"
            }
            ("unsupported", "alias@npm:create-example@1.2.3") => {
                "unsupported|alias@npm:create-example@1.2.3|0|0"
            }
            ("unsupported", "npm exec -- create-example@1.2.3") => {
                "unsupported|npm exec -- create-example@1.2.3|0|0"
            }
            ("unsupported", "npm exec --package create-example@1.2.3") => {
                "unsupported|npm exec --package create-example@1.2.3|0|0"
            }
            ("unsupported", "npm exec -c create-example@1.2.3") => {
                "unsupported|npm exec -c create-example@1.2.3|0|0"
            }
            ("unsupported", "npx --yes create-example@1.2.3") => {
                "unsupported|npx --yes create-example@1.2.3|0|0"
            }
            ("unsupported", "npm-exec create-example@1.2.3") => {
                "unsupported|npm-exec create-example@1.2.3|0|0"
            }
            ("unsupported", "create-example@1.2.3 other@2.0.0") => {
                "unsupported|create-example@1.2.3 other@2.0.0|0|0"
            }
            _ => unreachable!("manifest mapping must cover every no-network case"),
        }
    }
}
