//! Shared report redaction helpers for human and JSON output.

use serde::{Serialize, Serializer};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// Redact a string that may be displayed in a human report or JSON report.
pub fn redact_report_value(value: &str) -> String {
    redact_report_value_for_home(value, home_dir().as_deref())
}

/// Redact a report string using an explicit home path for deterministic tests.
pub(crate) fn redact_report_value_for_home(value: &str, home: Option<&Path>) -> String {
    let with_auth = redact_auth_like_value(value);
    let with_embedded_auth = redact_embedded_secret_assignments(&with_auth);
    let with_url = redact_url_credentials(&with_embedded_auth);
    redact_known_paths(&redact_inline_home(&with_url, home))
}

/// Build a stable digest key from an already-sanitized report value.
pub(crate) fn digest_report_key(label: &str, value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(label.as_bytes());
    hasher.update(b":");
    hasher.update(value.as_bytes());
    format!("sha256:{:x}", hasher.finalize())
}

/// Serde helper for report strings that may contain secrets or local paths.
pub fn serialize_redacted_string<S>(value: &String, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&redact_report_value(value))
}

/// Serde helper for optional report strings that may contain secrets or paths.
pub fn serialize_redacted_option_string<S>(
    value: &Option<String>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(value) => serializer.serialize_some(&redact_report_value(value)),
        None => serializer.serialize_none(),
    }
}

/// Serde helper for argv arrays that may contain secrets or local paths.
pub fn serialize_redacted_string_vec<S>(
    values: &Vec<String>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    redact_report_values(values).serialize(serializer)
}

/// Redact argv-like report values while preserving argument shape.
pub fn redact_report_values(values: &[String]) -> Vec<String> {
    let mut redact_next = false;
    values
        .iter()
        .map(|value| {
            if redact_next {
                redact_next = false;
                return "<redacted>".to_string();
            }
            let redacted = redact_report_value(value);
            if secret_arg_key(value) && !value.contains('=') {
                redact_next = true;
            }
            redacted
        })
        .collect()
}

/// Redact URL userinfo and token-like query parameters.
fn redact_url_credentials(url: &str) -> String {
    let Some(scheme_end) = url.find("://") else {
        return redact_query_secrets(url);
    };
    let authority_start = scheme_end + 3;
    let host_end = url[authority_start..]
        .find(['/', '?', '#'])
        .map(|offset| authority_start + offset)
        .unwrap_or(url.len());
    let without_userinfo = match url[authority_start..host_end].find('@') {
        Some(at_offset) => {
            let at = authority_start + at_offset;
            format!("{}<redacted>@{}", &url[..authority_start], &url[at + 1..])
        }
        None => url.to_string(),
    };
    redact_query_secrets(&without_userinfo)
}

fn redact_inline_home(value: &str, home: Option<&Path>) -> String {
    let Some(home) = home else {
        return value.to_string();
    };
    value.replace(home.to_string_lossy().as_ref(), "<home>")
}

fn redact_known_paths(value: &str) -> String {
    let redacted_users = redact_path_prefix(value, "/Users/");
    redact_path_prefix(&redacted_users, "/home/")
}

fn redact_path_prefix(value: &str, prefix: &str) -> String {
    let mut output = String::new();
    let mut rest = value;
    while let Some(index) = rest.find(prefix) {
        output.push_str(&rest[..index]);
        let path_start = index + prefix.len();
        let suffix = &rest[path_start..];
        let user_end = suffix
            .find(['/', ' ', '\n', '\t', '"', '\''])
            .unwrap_or(suffix.len());
        output.push_str("<home>");
        rest = &suffix[user_end..];
    }
    output.push_str(rest);
    output
}

fn redact_query_secrets(value: &str) -> String {
    let Some(query_index) = value.find('?') else {
        return value.to_string();
    };
    let (before_query, query_and_fragment) = value.split_at(query_index + 1);
    let (query, fragment) = query_and_fragment
        .split_once('#')
        .map(|(query, fragment)| (query, Some(fragment)))
        .unwrap_or((query_and_fragment, None));
    let redacted_query = query
        .split('&')
        .map(redact_query_pair)
        .collect::<Vec<_>>()
        .join("&");
    match fragment {
        Some(fragment) => format!("{before_query}{redacted_query}#{fragment}"),
        None => format!("{before_query}{redacted_query}"),
    }
}

fn redact_query_pair(pair: &str) -> String {
    let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
    if secret_key_name(key) {
        if pair.contains('=') {
            format!("{key}=<redacted>")
        } else {
            "<redacted>".to_string()
        }
    } else if pair.contains('=') {
        format!("{key}={value}")
    } else {
        key.to_string()
    }
}

fn redact_auth_like_value(value: &str) -> String {
    let lines = value.lines().map(redact_auth_like_line).collect::<Vec<_>>();
    if value.ends_with('\n') {
        format!("{}\n", lines.join("\n"))
    } else {
        lines.join("\n")
    }
}

fn redact_auth_like_line(line: &str) -> String {
    let trimmed = line.trim_start();
    let leading = &line[..line.len() - trimmed.len()];
    let lowered = trimmed.to_ascii_lowercase();
    if lowered.starts_with("authorization:") {
        return format!("{leading}Authorization: <redacted>");
    }
    if let Some((key, _)) = trimmed.split_once('=') {
        if secret_key_name(key) {
            return format!("{leading}{key}=<redacted>");
        }
    }
    if let Some((key, _)) = trimmed.split_once(':') {
        if secret_key_name(key) {
            return format!("{leading}{key}: <redacted>");
        }
    }
    line.to_string()
}

fn redact_embedded_secret_assignments(value: &str) -> String {
    let markers = [
        ":_authtoken=",
        "_authtoken=",
        "npm-auth-token=",
        "authtoken=",
        "token=",
        "password=",
        "secret=",
        "_auth=",
    ];
    let mut output = String::new();
    let mut cursor = 0;
    while let Some((start, marker_len)) = next_secret_assignment(value, cursor, &markers) {
        output.push_str(&value[cursor..start + marker_len]);
        output.push_str("<redacted>");
        cursor = secret_value_end(value, start + marker_len);
    }
    output.push_str(&value[cursor..]);
    output
}

fn next_secret_assignment(value: &str, cursor: usize, markers: &[&str]) -> Option<(usize, usize)> {
    let lowered = value[cursor..].to_ascii_lowercase();
    markers
        .iter()
        .filter_map(|marker| {
            lowered
                .find(marker)
                .map(|index| (cursor + index, marker.len()))
        })
        .min_by_key(|(index, _)| *index)
}

fn secret_value_end(value: &str, start: usize) -> usize {
    value[start..]
        .find(['&', ' ', '\n', '\t', '"', '\'', ';'])
        .map(|offset| start + offset)
        .unwrap_or(value.len())
}

fn secret_arg_key(value: &str) -> bool {
    secret_key_name(
        value
            .trim_start_matches('-')
            .split('=')
            .next()
            .unwrap_or(value),
    )
}

fn secret_key_name(key: &str) -> bool {
    let lowered = key.to_ascii_lowercase();
    lowered.contains("token")
        || lowered.contains("auth")
        || lowered.contains("secret")
        || lowered.contains("password")
        || lowered == "key"
        || lowered.ends_with("_key")
        || lowered.ends_with("-key")
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}
