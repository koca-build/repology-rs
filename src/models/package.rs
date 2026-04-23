use serde::{Deserialize, Serialize};

/// The version status of a package across repositories.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PackageStatus {
    Newest,
    Devel,
    Unique,
    Outdated,
    Legacy,
    Rolling,
    Noscheme,
    Incorrect,
    Untrusted,
    Ignored,
    /// A status not yet known to this library.
    Unknown(String),
}

impl<'de> Deserialize<'de> for PackageStatus {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "newest" => Self::Newest,
            "devel" => Self::Devel,
            "unique" => Self::Unique,
            "outdated" => Self::Outdated,
            "legacy" => Self::Legacy,
            "rolling" => Self::Rolling,
            "noscheme" => Self::Noscheme,
            "incorrect" => Self::Incorrect,
            "untrusted" => Self::Untrusted,
            "ignored" => Self::Ignored,
            _ => Self::Unknown(s),
        })
    }
}

impl Serialize for PackageStatus {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            Self::Newest => "newest",
            Self::Devel => "devel",
            Self::Unique => "unique",
            Self::Outdated => "outdated",
            Self::Legacy => "legacy",
            Self::Rolling => "rolling",
            Self::Noscheme => "noscheme",
            Self::Incorrect => "incorrect",
            Self::Untrusted => "untrusted",
            Self::Ignored => "ignored",
            Self::Unknown(s) => s.as_str(),
        };
        serializer.serialize_str(s)
    }
}

/// A single package entry from a repository, as returned by the Repology API.
///
/// Only `repo` and `version` are guaranteed to be present.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Package {
    /// Repository name (e.g., `"freebsd"`, `"arch"`).
    pub repo: String,

    /// Sanitized/normalized version string.
    pub version: String,

    /// Sub-repository (e.g., `"main"`, `"contrib"`, `"non-free"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subrepo: Option<String>,

    /// Source package name as used in the repository.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub srcname: Option<String>,

    /// Binary package name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binname: Option<String>,

    /// List of binary package names. Only populated by the single-project endpoint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binnames: Option<Vec<String>>,

    /// Human-readable package name for display purposes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visiblename: Option<String>,

    /// Original/raw version string from the repository. Can be `null`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origversion: Option<String>,

    /// Package version status relative to other repositories.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<PackageStatus>,

    /// One-line description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    /// Package categories.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<String>>,

    /// Package licenses.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub licenses: Option<Vec<String>>,

    /// Package maintainers (e.g., email addresses).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maintainers: Option<Vec<String>>,

    /// Whether this package version has known vulnerabilities.
    /// Only present in the API response when `true`.
    #[serde(default)]
    pub vulnerable: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_minimal_package() {
        let json = r#"{"repo": "freebsd", "version": "1.0"}"#;
        let pkg: Package = serde_json::from_str(json).unwrap();
        assert_eq!(pkg.repo, "freebsd");
        assert_eq!(pkg.version, "1.0");
        assert!(pkg.status.is_none());
        assert!(pkg.srcname.is_none());
        assert!(!pkg.vulnerable);
    }

    #[test]
    fn deserialize_full_package() {
        let json = r#"{
            "repo": "freebsd",
            "version": "50.1.0",
            "subrepo": "main",
            "srcname": "www/firefox",
            "binname": "firefox",
            "binnames": ["firefox", "firefox-esr"],
            "visiblename": "www/firefox",
            "origversion": "50.1.0_4,1",
            "status": "newest",
            "summary": "Widely used web browser",
            "categories": ["www"],
            "licenses": ["GPLv2+"],
            "maintainers": ["gecko@FreeBSD.org"],
            "vulnerable": true
        }"#;
        let pkg: Package = serde_json::from_str(json).unwrap();
        assert_eq!(pkg.status, Some(PackageStatus::Newest));
        assert_eq!(pkg.categories, Some(vec!["www".to_string()]));
        assert_eq!(
            pkg.binnames,
            Some(vec!["firefox".into(), "firefox-esr".into()])
        );
        assert!(pkg.vulnerable);
    }

    #[test]
    fn deserialize_null_origversion() {
        let json = r#"{"repo": "homebrew", "version": "1.0", "origversion": null}"#;
        let pkg: Package = serde_json::from_str(json).unwrap();
        assert_eq!(pkg.origversion, None);
    }

    #[test]
    fn deserialize_all_known_statuses() {
        for status in [
            "newest",
            "devel",
            "unique",
            "outdated",
            "legacy",
            "rolling",
            "noscheme",
            "incorrect",
            "untrusted",
            "ignored",
        ] {
            let json = format!(r#"{{"repo":"x","version":"1","status":"{}"}}"#, status);
            let pkg: Package = serde_json::from_str(&json).unwrap();
            assert!(
                !matches!(pkg.status, Some(PackageStatus::Unknown(_))),
                "status {status} should not be Unknown"
            );
        }
    }

    #[test]
    fn deserialize_unknown_status() {
        let json = r#"{"repo":"x","version":"1","status":"future_status"}"#;
        let pkg: Package = serde_json::from_str(json).unwrap();
        assert_eq!(
            pkg.status,
            Some(PackageStatus::Unknown("future_status".into()))
        );
    }

    #[test]
    fn status_round_trip() {
        let json = r#"{"repo":"x","version":"1","status":"newest"}"#;
        let pkg: Package = serde_json::from_str(json).unwrap();
        let serialized = serde_json::to_value(&pkg).unwrap();
        assert_eq!(serialized["status"], "newest");
    }

    #[test]
    fn unknown_status_round_trip() {
        let json = r#"{"repo":"x","version":"1","status":"future_thing"}"#;
        let pkg: Package = serde_json::from_str(json).unwrap();
        let serialized = serde_json::to_value(&pkg).unwrap();
        assert_eq!(serialized["status"], "future_thing");
    }

    #[test]
    fn vulnerable_absent_defaults_false() {
        let json = r#"{"repo":"x","version":"1"}"#;
        let pkg: Package = serde_json::from_str(json).unwrap();
        assert!(!pkg.vulnerable);
    }

    #[test]
    fn ignores_unknown_fields() {
        let json = r#"{"repo":"x","version":"1","some_new_field":"value"}"#;
        let pkg: Package = serde_json::from_str(json).unwrap();
        assert_eq!(pkg.repo, "x");
    }
}
