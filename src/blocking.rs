use std::collections::HashMap;
use std::time::Duration;

use crate::error::Result;
use crate::filter::ProjectFilter;
use crate::models::{Package, Problem};

/// Blocking (synchronous) client for the [Repology API](https://repology.org/api/v1).
///
/// Wraps the async [`RepologyClient`](crate::RepologyClient) with an internal
/// tokio runtime. All methods block the current thread until the request
/// completes.
///
/// # Examples
///
/// ```no_run
/// let client = repology::RepologyBlockingClient::new()?;
/// let packages = client.project("firefox")?;
/// println!("firefox has {} packages", packages.len());
/// # Ok::<(), repology::Error>(())
/// ```
pub struct RepologyBlockingClient {
    inner: crate::RepologyClient,
    rt: tokio::runtime::Runtime,
}

impl RepologyBlockingClient {
    /// Create a new blocking client with default settings.
    pub fn new() -> Result<Self> {
        Self::builder().build()
    }
}

#[bon::bon]
impl RepologyBlockingClient {
    /// Create a blocking client with custom configuration.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let client = repology::RepologyBlockingClient::builder()
    ///     .user_agent("my-app/1.0")
    ///     .build()?;
    /// # Ok::<(), repology::Error>(())
    /// ```
    #[builder]
    pub fn builder(
        #[builder(into, default = format!("repology-rs/{}", env!("CARGO_PKG_VERSION")))]
        user_agent: String,
        #[builder(into, default = "https://repology.org/api/v1".to_owned())] base_url: String,
        #[builder(default = Duration::from_secs(1))] rate_limit: Duration,
        reqwest_client: Option<reqwest::Client>,
        #[builder(default = 3)] max_retries: usize,
        #[builder(default = Duration::from_secs(1))] min_backoff: Duration,
        #[builder(default = Duration::from_secs(60))] max_backoff: Duration,
    ) -> Result<Self> {
        let inner = crate::RepologyClient::builder()
            .user_agent(user_agent)
            .base_url(base_url)
            .rate_limit(rate_limit)
            .maybe_reqwest_client(reqwest_client)
            .max_retries(max_retries)
            .min_backoff(min_backoff)
            .max_backoff(max_backoff)
            .build()?;

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| crate::error::Error::Config(format!("failed to create runtime: {e}")))?;

        Ok(Self { inner, rt })
    }
}

impl RepologyBlockingClient {
    /// Fetch all packages for a single project by name.
    pub fn project(&self, name: &str) -> Result<Vec<Package>> {
        self.rt.block_on(self.inner.project(name))
    }

    /// Fetch all projects matching the given filter, automatically paginating.
    pub fn projects(&self, filter: &ProjectFilter) -> Result<HashMap<String, Vec<Package>>> {
        self.rt.block_on(self.inner.projects(filter))
    }

    /// Fetch a single page of projects.
    pub fn projects_page(
        &self,
        filter: &ProjectFilter,
        cursor: Option<&str>,
    ) -> Result<HashMap<String, Vec<Package>>> {
        self.rt.block_on(self.inner.projects_page(filter, cursor))
    }

    /// Fetch all problems for a repository, automatically paginating.
    pub fn repository_problems(&self, repository: &str) -> Result<Vec<Problem>> {
        self.rt.block_on(self.inner.repository_problems(repository))
    }

    /// Fetch a single page of problems for a repository.
    pub fn repository_problems_page(
        &self,
        repository: &str,
        cursor: Option<&str>,
    ) -> Result<Vec<Problem>> {
        self.rt
            .block_on(self.inner.repository_problems_page(repository, cursor))
    }

    /// Fetch all problems for a maintainer in a repository, automatically
    /// paginating.
    pub fn maintainer_problems(&self, maintainer: &str, repository: &str) -> Result<Vec<Problem>> {
        self.rt
            .block_on(self.inner.maintainer_problems(maintainer, repository))
    }

    /// Fetch a single page of problems for a maintainer in a repository.
    pub fn maintainer_problems_page(
        &self,
        maintainer: &str,
        repository: &str,
        cursor: Option<&str>,
    ) -> Result<Vec<Problem>> {
        self.rt.block_on(
            self.inner
                .maintainer_problems_page(maintainer, repository, cursor),
        )
    }
}
