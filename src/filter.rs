/// Builder for query parameters when listing projects.
///
/// # Example
///
/// ```
/// use repology::ProjectFilter;
///
/// let filter = ProjectFilter::new()
///     .search("firefox")
///     .inrepo("archlinux")
///     .outdated(true);
/// ```
#[derive(Debug, Clone, Default)]
pub struct ProjectFilter {
    pub(crate) search: Option<String>,
    pub(crate) maintainer: Option<String>,
    pub(crate) category: Option<String>,
    pub(crate) inrepo: Option<String>,
    pub(crate) notinrepo: Option<String>,
    pub(crate) repos: Option<String>,
    pub(crate) families: Option<String>,
    pub(crate) repos_newest: Option<String>,
    pub(crate) families_newest: Option<String>,
    pub(crate) newest: Option<bool>,
    pub(crate) outdated: Option<bool>,
    pub(crate) problematic: Option<bool>,
}

impl ProjectFilter {
    /// Create a new empty filter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by project name substring.
    pub fn search(mut self, search: impl Into<String>) -> Self {
        self.search = Some(search.into());
        self
    }

    /// Filter by maintainer email.
    pub fn maintainer(mut self, maintainer: impl Into<String>) -> Self {
        self.maintainer = Some(maintainer.into());
        self
    }

    /// Filter by category.
    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Only include projects present in this repository.
    pub fn inrepo(mut self, repo: impl Into<String>) -> Self {
        self.inrepo = Some(repo.into());
        self
    }

    /// Exclude projects present in this repository.
    pub fn notinrepo(mut self, repo: impl Into<String>) -> Self {
        self.notinrepo = Some(repo.into());
        self
    }

    /// Filter by number of repositories (e.g., `"5"`, `"5-"`, `"-5"`, `"2-7"`).
    pub fn repos(mut self, repos: impl Into<String>) -> Self {
        self.repos = Some(repos.into());
        self
    }

    /// Filter by number of repository families.
    pub fn families(mut self, families: impl Into<String>) -> Self {
        self.families = Some(families.into());
        self
    }

    /// Filter by number of repos with newest version.
    pub fn repos_newest(mut self, repos_newest: impl Into<String>) -> Self {
        self.repos_newest = Some(repos_newest.into());
        self
    }

    /// Filter by number of families with newest version.
    pub fn families_newest(mut self, families_newest: impl Into<String>) -> Self {
        self.families_newest = Some(families_newest.into());
        self
    }

    /// Only include projects that have the newest version in some repo.
    pub fn newest(mut self, newest: bool) -> Self {
        self.newest = Some(newest);
        self
    }

    /// Only include outdated projects.
    pub fn outdated(mut self, outdated: bool) -> Self {
        self.outdated = Some(outdated);
        self
    }

    /// Only include projects with problems.
    pub fn problematic(mut self, problematic: bool) -> Self {
        self.problematic = Some(problematic);
        self
    }

    pub(crate) fn to_query_pairs(&self) -> Vec<(&str, &str)> {
        let mut pairs = Vec::new();
        if let Some(ref v) = self.search {
            pairs.push(("search", v.as_str()));
        }
        if let Some(ref v) = self.maintainer {
            pairs.push(("maintainer", v.as_str()));
        }
        if let Some(ref v) = self.category {
            pairs.push(("category", v.as_str()));
        }
        if let Some(ref v) = self.inrepo {
            pairs.push(("inrepo", v.as_str()));
        }
        if let Some(ref v) = self.notinrepo {
            pairs.push(("notinrepo", v.as_str()));
        }
        if let Some(ref v) = self.repos {
            pairs.push(("repos", v.as_str()));
        }
        if let Some(ref v) = self.families {
            pairs.push(("families", v.as_str()));
        }
        if let Some(ref v) = self.repos_newest {
            pairs.push(("repos_newest", v.as_str()));
        }
        if let Some(ref v) = self.families_newest {
            pairs.push(("families_newest", v.as_str()));
        }
        if self.newest == Some(true) {
            pairs.push(("newest", "1"));
        }
        if self.outdated == Some(true) {
            pairs.push(("outdated", "1"));
        }
        if self.problematic == Some(true) {
            pairs.push(("problematic", "1"));
        }
        pairs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_filter_produces_no_pairs() {
        let filter = ProjectFilter::new();
        assert!(filter.to_query_pairs().is_empty());
    }

    #[test]
    fn string_filters_produce_pairs() {
        let filter = ProjectFilter::new()
            .search("firefox")
            .inrepo("arch")
            .maintainer("test@example.com");
        let pairs = filter.to_query_pairs();
        assert!(pairs.contains(&("search", "firefox")));
        assert!(pairs.contains(&("inrepo", "arch")));
        assert!(pairs.contains(&("maintainer", "test@example.com")));
    }

    #[test]
    fn boolean_true_produces_pair() {
        let filter = ProjectFilter::new().outdated(true).newest(true);
        let pairs = filter.to_query_pairs();
        assert!(pairs.contains(&("outdated", "1")));
        assert!(pairs.contains(&("newest", "1")));
    }

    #[test]
    fn boolean_false_produces_no_pair() {
        let filter = ProjectFilter::new().outdated(false);
        let pairs = filter.to_query_pairs();
        assert!(!pairs.iter().any(|(k, _)| *k == "outdated"));
    }

    #[test]
    fn accepts_string_and_str() {
        let filter = ProjectFilter::new()
            .search("literal")
            .inrepo(String::from("owned"));
        let pairs = filter.to_query_pairs();
        assert!(pairs.contains(&("search", "literal")));
        assert!(pairs.contains(&("inrepo", "owned")));
    }
}
