use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The type of problem reported for a package.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ProblemType {
    HomepageDead,
    HomepagePermanentHttpsRedirect,
    HomepageDiscontinuedGoogle,
    HomepageDiscontinuedCodeplex,
    HomepageDiscontinuedGna,
    HomepageDiscontinuedCpan,
    HomepageSourceforgeMissingTrailingSlash,
    CpeUnreferenced,
    CpeMissing,
    DownloadDead,
    DownloadPermanentHttpsRedirect,
    /// A problem type not yet known to this library.
    Unknown(String),
}

impl<'de> Deserialize<'de> for ProblemType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "homepage_dead" => Self::HomepageDead,
            "homepage_permanent_https_redirect" => Self::HomepagePermanentHttpsRedirect,
            "homepage_discontinued_google" => Self::HomepageDiscontinuedGoogle,
            "homepage_discontinued_codeplex" => Self::HomepageDiscontinuedCodeplex,
            "homepage_discontinued_gna" => Self::HomepageDiscontinuedGna,
            "homepage_discontinued_cpan" => Self::HomepageDiscontinuedCpan,
            "homepage_sourceforge_missing_trailing_slash" => {
                Self::HomepageSourceforgeMissingTrailingSlash
            }
            "cpe_unreferenced" => Self::CpeUnreferenced,
            "cpe_missing" => Self::CpeMissing,
            "download_dead" => Self::DownloadDead,
            "download_permanent_https_redirect" => Self::DownloadPermanentHttpsRedirect,
            _ => Self::Unknown(s),
        })
    }
}

impl Serialize for ProblemType {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            Self::HomepageDead => "homepage_dead",
            Self::HomepagePermanentHttpsRedirect => "homepage_permanent_https_redirect",
            Self::HomepageDiscontinuedGoogle => "homepage_discontinued_google",
            Self::HomepageDiscontinuedCodeplex => "homepage_discontinued_codeplex",
            Self::HomepageDiscontinuedGna => "homepage_discontinued_gna",
            Self::HomepageDiscontinuedCpan => "homepage_discontinued_cpan",
            Self::HomepageSourceforgeMissingTrailingSlash => {
                "homepage_sourceforge_missing_trailing_slash"
            }
            Self::CpeUnreferenced => "cpe_unreferenced",
            Self::CpeMissing => "cpe_missing",
            Self::DownloadDead => "download_dead",
            Self::DownloadPermanentHttpsRedirect => "download_permanent_https_redirect",
            Self::Unknown(s) => s.as_str(),
        };
        serializer.serialize_str(s)
    }
}

/// A problem reported for a package in a repository.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Problem {
    /// Problem type identifier.
    #[serde(rename = "type")]
    pub problem_type: ProblemType,

    /// Additional structured data about the problem (e.g., URL, HTTP status code).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<HashMap<String, serde_json::Value>>,

    /// The Repology project name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_name: Option<String>,

    /// Normalized package version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Source package name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub srcname: Option<String>,

    /// Binary package name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binname: Option<String>,

    /// Raw version string from the repository.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rawversion: Option<String>,

    /// Package maintainer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maintainer: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_homepage_dead() {
        let json = r#"{
            "type": "homepage_dead",
            "data": {"code": 404, "url": "https://example.com"},
            "project_name": "test-project",
            "version": "1.0",
            "binname": "test",
            "srcname": "test-src",
            "rawversion": "1.0-1",
            "maintainer": "test@example.com"
        }"#;
        let problem: Problem = serde_json::from_str(json).unwrap();
        assert_eq!(problem.problem_type, ProblemType::HomepageDead);
        assert_eq!(problem.project_name, Some("test-project".into()));
        assert_eq!(problem.maintainer, Some("test@example.com".into()));

        let data = problem.data.unwrap();
        assert_eq!(data["code"], 404);
        assert_eq!(data["url"], "https://example.com");
    }

    #[test]
    fn deserialize_all_known_problem_types() {
        let types = [
            "homepage_dead",
            "homepage_permanent_https_redirect",
            "homepage_discontinued_google",
            "homepage_discontinued_codeplex",
            "homepage_discontinued_gna",
            "homepage_discontinued_cpan",
            "homepage_sourceforge_missing_trailing_slash",
            "cpe_unreferenced",
            "cpe_missing",
            "download_dead",
            "download_permanent_https_redirect",
        ];
        for t in types {
            let json = format!(r#"{{"type":"{}"}}"#, t);
            let problem: Problem = serde_json::from_str(&json).unwrap();
            assert!(
                !matches!(problem.problem_type, ProblemType::Unknown(_)),
                "type {t} should not be Unknown"
            );
        }
    }

    #[test]
    fn deserialize_unknown_problem_type() {
        let json = r#"{"type": "new_future_problem"}"#;
        let problem: Problem = serde_json::from_str(json).unwrap();
        assert_eq!(
            problem.problem_type,
            ProblemType::Unknown("new_future_problem".into())
        );
    }

    #[test]
    fn problem_type_round_trip() {
        let json = r#"{"type": "homepage_dead"}"#;
        let problem: Problem = serde_json::from_str(json).unwrap();
        let serialized = serde_json::to_value(&problem).unwrap();
        assert_eq!(serialized["type"], "homepage_dead");
    }

    #[test]
    fn unknown_problem_type_round_trip() {
        let json = r#"{"type": "future_thing"}"#;
        let problem: Problem = serde_json::from_str(json).unwrap();
        let serialized = serde_json::to_value(&problem).unwrap();
        assert_eq!(serialized["type"], "future_thing");
    }

    #[test]
    fn ignores_unknown_fields() {
        let json = r#"{"type": "homepage_dead", "new_field": 42}"#;
        let problem: Problem = serde_json::from_str(json).unwrap();
        assert_eq!(problem.problem_type, ProblemType::HomepageDead);
    }
}
