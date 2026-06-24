//! Local package-spec parsing for the M1 exact-version resolver surface.

use crate::{
    CommandIntent, M1Reason, MalformedSpec, PackageSpec, PackageSpecParse, UnsupportedSpec,
    UnsupportedSpecCategory,
};

/// Parse the M1 command intent without touching registry or artifact code.
pub fn parse_command_intent(raw_spec: &str, forwarded_args: Vec<String>) -> CommandIntent {
    match classify_package_spec(raw_spec) {
        PackageSpecParse::Supported(package_spec) => {
            CommandIntent::supported(package_spec, forwarded_args)
        }
        PackageSpecParse::Unsupported(unsupported) => {
            CommandIntent::unsupported(raw_spec.to_string(), unsupported)
        }
        PackageSpecParse::Malformed(malformed) => {
            CommandIntent::malformed(raw_spec.to_string(), malformed)
        }
    }
}

/// Classify a package spec inside the M1 supported/unsupported matrix.
pub fn classify_package_spec(raw_spec: &str) -> PackageSpecParse {
    let spec = raw_spec.trim();
    if spec.is_empty() {
        return malformed(raw_spec);
    }

    if is_npm_exec_variant(spec) {
        return unsupported(UnsupportedSpecCategory::NpmExecVariant);
    }

    if spec.split_whitespace().count() > 1 {
        return unsupported(UnsupportedSpecCategory::MultipleSpecs);
    }

    if is_tarball_url(spec) {
        return unsupported(UnsupportedSpecCategory::TarballUrl);
    }

    if is_git_spec(spec) {
        return unsupported(UnsupportedSpecCategory::GitUrl);
    }

    if is_local_path(spec) {
        return unsupported(UnsupportedSpecCategory::LocalPath);
    }

    if is_alias_spec(spec) {
        return unsupported(UnsupportedSpecCategory::Alias);
    }

    if is_unversioned_name(spec) {
        return unsupported(UnsupportedSpecCategory::UnversionedName);
    }

    parse_exact_package_spec(spec).unwrap_or_else(|| malformed(raw_spec))
}

/// Parse an unscoped or scoped package spec that appears to include a version.
fn parse_exact_package_spec(spec: &str) -> Option<PackageSpecParse> {
    if spec.starts_with('@') {
        return parse_scoped_exact_package_spec(spec);
    }

    let (name, version) = spec.rsplit_once('@')?;
    if name.is_empty() {
        return None;
    }
    if version.is_empty() {
        return None;
    }
    if name.contains('/') {
        return Some(unsupported(UnsupportedSpecCategory::Other));
    }
    if !is_exact_semver(version) {
        return Some(unsupported(UnsupportedSpecCategory::VersionRange));
    }

    Some(PackageSpecParse::Supported(PackageSpec::exact(
        spec, name, version, None,
    )))
}

/// Parse `@scope/name@version` specs inside the M1 supported surface.
fn parse_scoped_exact_package_spec(spec: &str) -> Option<PackageSpecParse> {
    let (name, version) = spec.rsplit_once('@')?;
    if version.is_empty() {
        return None;
    }
    let (scope, package) = name.strip_prefix('@')?.split_once('/')?;
    if scope.is_empty() || package.is_empty() {
        return None;
    }
    if package.contains('/') {
        return Some(unsupported(UnsupportedSpecCategory::Other));
    }
    if !is_exact_semver(version) {
        return Some(unsupported(UnsupportedSpecCategory::VersionRange));
    }

    Some(PackageSpecParse::Supported(PackageSpec::exact(
        spec,
        name,
        version,
        Some(scope.to_string()),
    )))
}

/// Build a fail-closed unsupported-spec classification.
fn unsupported(category: UnsupportedSpecCategory) -> PackageSpecParse {
    PackageSpecParse::Unsupported(UnsupportedSpec {
        reason: M1Reason::UnsupportedSpec,
        category,
        downloaded: false,
    })
}

/// Build a fail-closed malformed-spec classification.
fn malformed(raw: &str) -> PackageSpecParse {
    PackageSpecParse::Malformed(MalformedSpec {
        reason: M1Reason::MalformedSpec,
        raw: raw.to_string(),
        downloaded: false,
    })
}

/// Return true when a version token is an exact SemVer version.
fn is_exact_semver(version: &str) -> bool {
    let (without_build, build) = split_once_optional(version, '+');
    if let Some(build) = build {
        if !valid_dot_identifiers(build, false) {
            return false;
        }
    }

    let (core, prerelease) = split_once_optional(without_build, '-');
    if let Some(prerelease) = prerelease {
        if !valid_dot_identifiers(prerelease, true) {
            return false;
        }
    }

    let mut core_parts = core.split('.');
    let major = core_parts.next();
    let minor = core_parts.next();
    let patch = core_parts.next();
    if core_parts.next().is_some() {
        return false;
    }

    matches!(
        (major, minor, patch),
        (Some(major), Some(minor), Some(patch))
            if valid_numeric_identifier(major)
                && valid_numeric_identifier(minor)
                && valid_numeric_identifier(patch)
    )
}

/// Split once on a separator and keep the missing suffix explicit.
fn split_once_optional(input: &str, separator: char) -> (&str, Option<&str>) {
    input
        .split_once(separator)
        .map_or((input, None), |(head, tail)| (head, Some(tail)))
}

/// Validate dot-separated prerelease or build identifiers.
fn valid_dot_identifiers(value: &str, enforce_numeric_leading_zero: bool) -> bool {
    !value.is_empty()
        && value.split('.').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
                && (!enforce_numeric_leading_zero
                    || !part.bytes().all(|byte| byte.is_ascii_digit())
                    || valid_numeric_identifier(part))
        })
}

/// Validate a SemVer numeric identifier.
fn valid_numeric_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| byte.is_ascii_digit())
        && (value == "0" || !value.starts_with('0'))
}

/// Return true when the input looks like an unsupported npm execution command.
fn is_npm_exec_variant(spec: &str) -> bool {
    spec == "npm"
        || spec == "npx"
        || spec == "npm-exec"
        || spec.starts_with("npm ")
        || spec.starts_with("npx ")
        || spec.starts_with("npm-exec ")
}

/// Return true when the input is a direct tarball URL.
fn is_tarball_url(spec: &str) -> bool {
    (spec.starts_with("https://") || spec.starts_with("http://")) && spec.ends_with(".tgz")
}

/// Return true when the input is a git dependency spec.
fn is_git_spec(spec: &str) -> bool {
    spec.starts_with("git+")
        || spec.starts_with("git://")
        || spec.starts_with("github:")
        || spec.starts_with("gitlab:")
        || spec.starts_with("bitbucket:")
        || spec.ends_with(".git")
}

/// Return true when the input is a local filesystem path.
fn is_local_path(spec: &str) -> bool {
    spec == "."
        || spec == ".."
        || spec.starts_with("./")
        || spec.starts_with("../")
        || spec.starts_with(".\\")
        || spec.starts_with("..\\")
        || spec.starts_with('/')
        || spec.contains('\\')
        || spec.starts_with("file:")
        || is_windows_drive_path(spec)
}

/// Return true when the input looks like a Windows drive path.
fn is_windows_drive_path(spec: &str) -> bool {
    let bytes = spec.as_bytes();
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'\\' || bytes[2] == b'/')
}

/// Return true when the input is an npm alias spec.
fn is_alias_spec(spec: &str) -> bool {
    spec.contains("npm:")
}

/// Return true when the input is a package name without a version token.
fn is_unversioned_name(spec: &str) -> bool {
    if spec.starts_with('@') {
        let Some((scope, package)) = spec[1..].split_once('/') else {
            return false;
        };

        return !spec[1..].contains('@') && !scope.is_empty() && !package.is_empty();
    }

    !spec.contains('@')
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies the supported unscoped exact-version parser path.
    #[test]
    fn parses_unscoped_exact_version_specs() {
        let intent = parse_command_intent(
            "create-example@1.2.3",
            vec!["--template".to_string(), "react".to_string()],
        );

        assert_eq!(intent.forwarded_args, ["--template", "react"]);
        let spec = match intent.package_spec {
            PackageSpecParse::Supported(spec) => Some(spec),
            _ => None,
        };
        assert!(spec.is_some());
        let spec = spec.expect("supported spec was asserted");
        assert_eq!(spec.name, "create-example");
        assert_eq!(spec.version, "1.2.3");
        assert_eq!(spec.scope, None);
    }

    /// Verifies the supported scoped exact-version parser path.
    #[test]
    fn parses_scoped_exact_version_specs() {
        let intent = parse_command_intent("@scope/create-example@1.2.3-beta.1", Vec::new());

        let spec = match intent.package_spec {
            PackageSpecParse::Supported(spec) => Some(spec),
            _ => None,
        };
        assert!(spec.is_some());
        let spec = spec.expect("supported scoped spec was asserted");
        assert_eq!(spec.name, "@scope/create-example");
        assert_eq!(spec.version, "1.2.3-beta.1");
        assert_eq!(spec.scope, Some("scope".to_string()));
    }

    /// Verifies exact versions can include prerelease/build identifiers.
    #[test]
    fn parses_exact_semver_prerelease_and_build_versions() {
        for raw in [
            "create-example@1.2.3-next.0",
            "create-example@1.2.3+exp.sha",
            "npm-exec-helper@1.2.3",
        ] {
            let intent = parse_command_intent(raw, Vec::new());
            assert!(matches!(
                intent.package_spec,
                PackageSpecParse::Supported(_)
            ));
        }
    }

    /// Verifies unsupported specs fail closed before network-capable work.
    #[test]
    fn rejects_unsupported_specs_without_downloads() {
        let cases = [
            ("create-example", UnsupportedSpecCategory::UnversionedName),
            (
                "create-example@latest",
                UnsupportedSpecCategory::VersionRange,
            ),
            (
                "create-example@^1.2.3",
                UnsupportedSpecCategory::VersionRange,
            ),
            ("create-example@beta", UnsupportedSpecCategory::VersionRange),
            ("create-example@next", UnsupportedSpecCategory::VersionRange),
            ("create-example@1", UnsupportedSpecCategory::VersionRange),
            ("create-example@1.2", UnsupportedSpecCategory::VersionRange),
            ("github:user/repo", UnsupportedSpecCategory::GitUrl),
            ("./local-package", UnsupportedSpecCategory::LocalPath),
            ("..\\local@1.2.3", UnsupportedSpecCategory::LocalPath),
            ("C:\\tmp\\pkg@1.2.3", UnsupportedSpecCategory::LocalPath),
            (
                "https://example.test/pkg.tgz",
                UnsupportedSpecCategory::TarballUrl,
            ),
            (
                "alias@npm:create-example@1.2.3",
                UnsupportedSpecCategory::Alias,
            ),
            (
                "npm exec create-example@1.2.3",
                UnsupportedSpecCategory::NpmExecVariant,
            ),
        ];

        for (raw, expected_category) in cases {
            let intent = parse_command_intent(raw, vec!["--ignored".to_string()]);
            assert!(intent.forwarded_args.is_empty(), "{raw}");
            let unsupported = match intent.package_spec {
                PackageSpecParse::Unsupported(unsupported) => Some(unsupported),
                _ => None,
            };
            assert!(unsupported.is_some(), "{raw}");
            let unsupported = unsupported.expect("unsupported spec was asserted");
            assert_eq!(unsupported.reason, M1Reason::UnsupportedSpec, "{raw}");
            assert_eq!(unsupported.category, expected_category, "{raw}");
            assert!(!unsupported.downloaded, "{raw}");
        }
    }

    /// Verifies malformed specs remain local parser failures.
    #[test]
    fn rejects_malformed_specs_without_downloads() {
        for raw in ["", "@scope/", "create-example@", "@/pkg@1.2.3"] {
            let intent = parse_command_intent(raw, vec!["--ignored".to_string()]);
            assert!(intent.forwarded_args.is_empty(), "{raw}");
            let malformed = match intent.package_spec {
                PackageSpecParse::Malformed(malformed) => Some(malformed),
                _ => None,
            };
            assert!(malformed.is_some(), "{raw}");
            let malformed = malformed.expect("malformed spec was asserted");
            assert_eq!(malformed.reason, M1Reason::MalformedSpec, "{raw}");
            assert!(!malformed.downloaded, "{raw}");
        }
    }
}
