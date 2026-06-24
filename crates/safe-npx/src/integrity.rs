//! npm integrity verification and root artifact identity.

use crate::{ArtifactBytes, ArtifactIdentity, Decision, M1Reason};
use base64::prelude::{Engine as _, BASE64_STANDARD};
use sha2::{Digest, Sha512};

/// Supported artifact digest algorithm for M1 identity.
pub const ARTIFACT_DIGEST_ALGORITHM: &str = "sha512";

/// Error returned when artifact bytes cannot be verified.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactVerificationError {
    /// Policy decision for the failed verification.
    pub decision: Decision,
    /// Stable M1 reason for the failed verification.
    pub reason: M1Reason,
    /// Short diagnostic detail for logs and tests.
    pub detail: String,
}

impl ArtifactVerificationError {
    /// Create an integrity mismatch denial.
    pub fn integrity_mismatch(detail: impl Into<String>) -> Self {
        Self {
            decision: Decision::Deny,
            reason: M1Reason::IntegrityMismatch,
            detail: detail.into(),
        }
    }
}

/// Verify npm integrity metadata and return stable artifact identity.
pub fn verify_artifact_integrity(
    artifact_bytes: &ArtifactBytes,
    integrity: &str,
) -> Result<ArtifactIdentity, ArtifactVerificationError> {
    let expected_digest = parse_sha512_integrity(integrity)?;
    let actual_digest = sha512_digest(&artifact_bytes.bytes);

    if expected_digest != actual_digest {
        return Err(ArtifactVerificationError::integrity_mismatch(
            "downloaded bytes did not match integrity metadata",
        ));
    }

    Ok(ArtifactIdentity {
        name: artifact_bytes.name.clone(),
        version: artifact_bytes.version.clone(),
        integrity: integrity.to_string(),
        digest_algorithm: ARTIFACT_DIGEST_ALGORITHM.to_string(),
        digest: hex_digest(&actual_digest),
    })
}

/// Parse a supported npm sha512 integrity string.
fn parse_sha512_integrity(integrity: &str) -> Result<Vec<u8>, ArtifactVerificationError> {
    if integrity.trim().is_empty() {
        return Err(ArtifactVerificationError::integrity_mismatch(
            "integrity metadata is missing",
        ));
    }

    let (algorithm, digest) = integrity.split_once('-').ok_or_else(|| {
        ArtifactVerificationError::integrity_mismatch("integrity metadata is malformed")
    })?;
    if algorithm != ARTIFACT_DIGEST_ALGORITHM {
        return Err(ArtifactVerificationError::integrity_mismatch(
            "integrity algorithm is unsupported",
        ));
    }
    if digest.is_empty() {
        return Err(ArtifactVerificationError::integrity_mismatch(
            "integrity digest is missing",
        ));
    }

    BASE64_STANDARD.decode(digest).map_err(|error| {
        ArtifactVerificationError::integrity_mismatch(format!(
            "integrity digest is not valid base64: {error}"
        ))
    })
}

/// Compute the raw SHA-512 digest for artifact bytes.
fn sha512_digest(bytes: &[u8]) -> Vec<u8> {
    Sha512::digest(bytes).to_vec()
}

/// Encode digest bytes as lowercase hexadecimal.
fn hex_digest(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join("")
}

#[cfg(test)]
/// Tests for npm integrity verification and artifact identity.
mod tests {
    use super::*;

    #[test]
    /// Verifies valid integrity returns a stable artifact identity.
    fn verifies_valid_sha512_integrity() {
        let artifact = artifact_bytes(b"fixture-tarball");
        let integrity = integrity_for(&artifact.bytes);

        let identity = verify_artifact_integrity(&artifact, &integrity)
            .expect("valid integrity should verify");

        assert_eq!(identity.name, "create-example");
        assert_eq!(identity.version, "1.2.3");
        assert_eq!(identity.integrity, integrity);
        assert_eq!(identity.digest_algorithm, ARTIFACT_DIGEST_ALGORITHM);
        assert_eq!(identity.digest, hex_digest(&sha512_digest(&artifact.bytes)));
    }

    #[test]
    /// Verifies repeated verification of identical bytes is deterministic.
    fn digest_identity_is_stable() {
        let artifact = artifact_bytes(b"fixture-tarball");
        let integrity = integrity_for(&artifact.bytes);

        let first = verify_artifact_integrity(&artifact, &integrity)
            .expect("first verification should pass");
        let second = verify_artifact_integrity(&artifact, &integrity)
            .expect("second verification should pass");

        assert_eq!(first, second);
    }

    #[test]
    /// Verifies mismatched integrity denies execution.
    fn denies_integrity_mismatch() {
        let artifact = artifact_bytes(b"fixture-tarball");
        let wrong_integrity = integrity_for(b"different-bytes");

        assert_denied_integrity_mismatch(verify_artifact_integrity(&artifact, &wrong_integrity));
    }

    #[test]
    /// Verifies missing integrity metadata denies execution.
    fn denies_missing_integrity() {
        assert_denied_integrity_mismatch(verify_artifact_integrity(
            &artifact_bytes(b"fixture-tarball"),
            "",
        ));
    }

    #[test]
    /// Verifies malformed integrity metadata denies execution.
    fn denies_malformed_integrity() {
        assert_denied_integrity_mismatch(verify_artifact_integrity(
            &artifact_bytes(b"fixture-tarball"),
            "sha512:not-a-valid-sri",
        ));
    }

    #[test]
    /// Verifies unsupported integrity algorithms deny execution.
    fn denies_unsupported_integrity_algorithm() {
        assert_denied_integrity_mismatch(verify_artifact_integrity(
            &artifact_bytes(b"fixture-tarball"),
            "sha1-YWJj",
        ));
    }

    /// Build downloaded artifact bytes for tests.
    fn artifact_bytes(bytes: &[u8]) -> ArtifactBytes {
        ArtifactBytes {
            name: "create-example".to_string(),
            version: "1.2.3".to_string(),
            tarball_url: "https://registry.npmjs.org/create-example/-/create-example-1.2.3.tgz"
                .to_string(),
            bytes: bytes.to_vec(),
        }
    }

    /// Return npm sha512 integrity metadata for bytes.
    fn integrity_for(bytes: &[u8]) -> String {
        format!(
            "{}-{}",
            ARTIFACT_DIGEST_ALGORITHM,
            BASE64_STANDARD.encode(sha512_digest(bytes))
        )
    }

    /// Assert verification failed with deny and integrity_mismatch.
    fn assert_denied_integrity_mismatch(
        result: Result<ArtifactIdentity, ArtifactVerificationError>,
    ) {
        let error = result.expect_err("verification should fail");

        assert_eq!(error.decision, Decision::Deny);
        assert_eq!(error.reason, M1Reason::IntegrityMismatch);
    }
}
