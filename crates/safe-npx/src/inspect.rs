//! Inspect boundary that proves parser failures stop before network hooks.

use crate::{CommandIntent, PackageSpec, PackageSpecParse};

/// Network-capable hooks used after parser classification.
pub trait NetworkProbe {
    /// Record or perform a registry metadata lookup for a supported spec.
    fn resolve_metadata(&mut self, package_spec: &PackageSpec);

    /// Record or perform a tarball download for a supported spec.
    fn download_tarball(&mut self, package_spec: &PackageSpec);
}

/// Inspect parsed intent and only enter network-capable hooks for supported specs.
pub fn inspect_with_probe(intent: &CommandIntent, probe: &mut impl NetworkProbe) {
    if let PackageSpecParse::Supported(package_spec) = &intent.package_spec {
        probe.resolve_metadata(package_spec);
        probe.download_tarball(package_spec);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_command_intent;

    #[derive(Default)]
    struct CountingProbe {
        registry_calls: usize,
        tarball_calls: usize,
    }

    impl NetworkProbe for CountingProbe {
        fn resolve_metadata(&mut self, _package_spec: &PackageSpec) {
            self.registry_calls += 1;
        }

        fn download_tarball(&mut self, _package_spec: &PackageSpec) {
            self.tarball_calls += 1;
        }
    }

    #[test]
    fn malformed_specs_do_not_reach_network_hooks() {
        for raw in ["", "@scope/", "create-example@", "@/pkg@1.2.3"] {
            let intent = parse_command_intent(raw, Vec::new());
            let mut probe = CountingProbe::default();

            inspect_with_probe(&intent, &mut probe);

            assert_eq!(probe.registry_calls, 0, "{raw}");
            assert_eq!(probe.tarball_calls, 0, "{raw}");
        }
    }

    #[test]
    fn unsupported_specs_do_not_reach_network_hooks() {
        for raw in [
            "create-example",
            "create-example@latest",
            "create-example@^1.2.3",
            "github:user/repo",
            "./local-package",
            "C:tmp@1.2.3",
            "https://example.test/pkg.tgz",
            "alias@npm:create-example@1.2.3",
            "npm exec -- create-example@1.2.3",
            "create-example@1.2.3 other@2.0.0",
        ] {
            let intent = parse_command_intent(raw, Vec::new());
            let mut probe = CountingProbe::default();

            inspect_with_probe(&intent, &mut probe);

            assert_eq!(probe.registry_calls, 0, "{raw}");
            assert_eq!(probe.tarball_calls, 0, "{raw}");
        }
    }

    #[test]
    fn supported_specs_enter_network_hooks() {
        let intent = parse_command_intent("create-example@1.2.3", Vec::new());
        let mut probe = CountingProbe::default();

        inspect_with_probe(&intent, &mut probe);

        assert_eq!(probe.registry_calls, 1);
        assert_eq!(probe.tarball_calls, 1);
    }
}
