use std::collections::HashMap;
use std::pin::Pin;
use std::time::{Duration, Instant};

use backon::{ExponentialBuilder, Retryable};
use futures_core::Stream;
use tokio::sync::Mutex;

type BoxStream<'a, T> = Pin<Box<dyn Stream<Item = T> + 'a>>;

use crate::error::{Error, Result};
use crate::filter::ProjectFilter;
use crate::models::{Package, Problem};

const DEFAULT_BASE_URL: &str = "https://repology.org/api/v1";

/// Configuration for automatic retry with exponential backoff.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (not counting the initial attempt).
    /// Set to 0 to disable retries.
    pub max_retries: usize,
    /// Initial backoff duration before the first retry.
    pub min_backoff: Duration,
    /// Maximum backoff duration cap.
    pub max_backoff: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            min_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(60),
        }
    }
}

/// Async client for the [Repology API](https://repology.org/api/v1).
///
/// Constructed via [`RepologyClient::new`] or [`RepologyClient::builder`].
/// Enforces rate limiting (1 request/sec by default) as required by the
/// Repology API policy.
pub struct RepologyClient {
    http: reqwest::Client,
    base_url: String,
    rate_limit: Duration,
    last_request: Mutex<Option<Instant>>,
    retry_config: RetryConfig,
}

impl RepologyClient {
    /// Create a new client with default settings.
    ///
    /// Uses a User-Agent of `repology-rs/{version}`, the default base URL,
    /// and 1 request/sec rate limiting.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn example() -> repology::Result<()> {
    /// let client = repology::RepologyClient::new()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new() -> Result<Self> {
        Self::builder().build()
    }
}

#[bon::bon]
impl RepologyClient {
    /// Create a client with custom configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn example() -> repology::Result<()> {
    /// let client = repology::RepologyClient::builder()
    ///     .user_agent("my-app/1.0 (https://github.com/me/my-app)")
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    #[builder]
    pub fn builder(
        #[builder(into, default = format!("repology-rs/{}", env!("CARGO_PKG_VERSION")))]
        user_agent: String,
        #[builder(into, default = DEFAULT_BASE_URL.to_owned())] base_url: String,
        #[builder(default = Duration::from_secs(1))] rate_limit: Duration,
        reqwest_client: Option<reqwest::Client>,
        #[builder(default = 3)] max_retries: usize,
        #[builder(default = Duration::from_secs(1))] min_backoff: Duration,
        #[builder(default = Duration::from_secs(60))] max_backoff: Duration,
    ) -> Result<Self> {
        if user_agent.is_empty() {
            return Err(Error::Config(
                "user_agent must not be empty per Repology API policy".into(),
            ));
        }

        let http = match reqwest_client {
            Some(c) => c,
            None => reqwest::Client::builder()
                .user_agent(&user_agent)
                .build()
                .map_err(Error::Http)?,
        };

        let retry_config = RetryConfig {
            max_retries,
            min_backoff,
            max_backoff,
        };

        Ok(Self {
            http,
            base_url,
            rate_limit,
            last_request: Mutex::new(None),
            retry_config,
        })
    }
}

impl RepologyClient {
    // ── Internal helpers ───────────────────────────────────────────

    async fn rate_limit(&self) {
        if self.rate_limit.is_zero() {
            return;
        }
        let mut last = self.last_request.lock().await;
        if let Some(t) = *last {
            let elapsed = t.elapsed();
            if elapsed < self.rate_limit {
                tokio::time::sleep(self.rate_limit - elapsed).await;
            }
        }
        *last = Some(Instant::now());
    }

    async fn get(&self, url: &str) -> Result<reqwest::Response> {
        let backoff = ExponentialBuilder::new()
            .with_min_delay(self.retry_config.min_backoff)
            .with_max_delay(self.retry_config.max_backoff)
            .with_max_times(self.retry_config.max_retries)
            .with_jitter();

        (|| self.get_once(url))
            .retry(backoff)
            .when(is_retryable)
            .await
    }

    async fn get_once(&self, url: &str) -> Result<reqwest::Response> {
        self.rate_limit().await;
        let resp = self.http.get(url).send().await.map_err(Error::Http)?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(Error::Api { status, body });
        }

        Ok(resp)
    }

    async fn get_json<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T> {
        let resp = self.get(url).await?;
        let body = resp.text().await.map_err(Error::Http)?;
        serde_json::from_str(&body).map_err(|e| Error::Deserialize { source: e, body })
    }

    fn build_projects_url(&self, cursor: Option<&str>, filter: &ProjectFilter) -> String {
        let path = match cursor {
            None => format!("{}/projects/", self.base_url),
            Some(name) => format!("{}/projects/{}/", self.base_url, urlencoding(name)),
        };

        let pairs = filter.to_query_pairs();
        if pairs.is_empty() {
            return path;
        }

        let mut url = url::Url::parse(&path).expect("base_url + path should be valid");
        for (key, value) in &pairs {
            url.query_pairs_mut().append_pair(key, value);
        }
        url.to_string()
    }

    // ── Project endpoints ─────────────────────────────────────────

    /// Fetch all packages for a single project by name.
    pub async fn project(&self, name: &str) -> Result<Vec<Package>> {
        let url = format!("{}/project/{}", self.base_url, urlencoding(name));
        self.get_json(&url).await
    }

    /// Fetch all projects matching the given filter, automatically paginating
    /// through every page.
    ///
    /// For large result sets, prefer [`projects_iter`](Self::projects_iter)
    /// to avoid loading everything into memory at once.
    pub async fn projects(&self, filter: &ProjectFilter) -> Result<HashMap<String, Vec<Package>>> {
        use tokio_stream::StreamExt;
        let mut all = HashMap::new();
        let mut stream = self.projects_iter(filter);
        while let Some(result) = stream.next().await {
            let (name, packages) = result?;
            all.insert(name, packages);
        }
        Ok(all)
    }

    /// Returns a [`Stream`] that automatically paginates through all projects
    /// matching the given filter, yielding `(project_name, packages)` pairs.
    ///
    /// Uses [`projects_page`](Self::projects_page) under the hood.
    pub fn projects_iter<'a>(
        &'a self,
        filter: &'a ProjectFilter,
    ) -> BoxStream<'a, Result<(String, Vec<Package>)>> {
        Box::pin(async_stream::try_stream! {
            let mut cursor: Option<String> = None;

            loop {
                let page = self.projects_page(filter, cursor.as_deref()).await?;

                if page.is_empty() {
                    break;
                }

                let mut entries: Vec<(String, Vec<Package>)> = page.into_iter().collect();
                entries.sort_by(|a, b| a.0.cmp(&b.0));

                let last_name = entries.last().map(|(name, _)| name.clone());
                let is_last_page = entries.len() < 200;

                for (name, packages) in entries {
                    if Some(name.as_str()) == cursor.as_deref() {
                        continue;
                    }
                    yield (name, packages);
                }

                if is_last_page {
                    break;
                }

                cursor = last_name;
            }
        })
    }

    /// Fetch a single page of projects (up to ~200).
    ///
    /// Pass `None` for the first page, then pass the last project name from
    /// the previous page as the cursor to get the next page.
    pub async fn projects_page(
        &self,
        filter: &ProjectFilter,
        cursor: Option<&str>,
    ) -> Result<HashMap<String, Vec<Package>>> {
        let url = self.build_projects_url(cursor, filter);
        self.get_json(&url).await
    }

    // ── Problem endpoints ─────────────────────────────────────────

    /// Fetch all problems for a repository, automatically paginating.
    ///
    /// For large result sets, prefer
    /// [`repository_problems_iter`](Self::repository_problems_iter).
    pub async fn repository_problems(&self, repository: &str) -> Result<Vec<Problem>> {
        use tokio_stream::StreamExt;
        let mut all = Vec::new();
        let mut stream = self.repository_problems_iter(repository);
        while let Some(result) = stream.next().await {
            all.push(result?);
        }
        Ok(all)
    }

    /// Returns a [`Stream`] that automatically paginates through all problems
    /// for a repository.
    ///
    /// Uses [`repository_problems_page`](Self::repository_problems_page)
    /// under the hood.
    pub fn repository_problems_iter<'a>(
        &'a self,
        repository: &'a str,
    ) -> BoxStream<'a, Result<Problem>> {
        Box::pin(async_stream::try_stream! {
            let mut cursor: Option<String> = None;

            loop {
                let page = self.repository_problems_page(repository, cursor.as_deref()).await?;

                if page.is_empty() {
                    break;
                }

                cursor = page.last().and_then(|p| p.project_name.clone());

                for problem in page {
                    yield problem;
                }

                if cursor.is_none() {
                    break;
                }
            }
        })
    }

    /// Fetch a single page of problems for a repository.
    ///
    /// Pass `None` for the first page, then pass the last `project_name` from
    /// the previous page as the cursor to get the next page.
    pub async fn repository_problems_page(
        &self,
        repository: &str,
        cursor: Option<&str>,
    ) -> Result<Vec<Problem>> {
        let mut url = format!(
            "{}/repository/{}/problems",
            self.base_url,
            urlencoding(repository),
        );
        if let Some(start) = cursor {
            url.push_str(&format!("?start={}", urlencoding(start)));
        }
        self.get_json(&url).await
    }

    /// Fetch all problems for a maintainer in a repository, automatically
    /// paginating.
    ///
    /// For large result sets, prefer
    /// [`maintainer_problems_iter`](Self::maintainer_problems_iter).
    pub async fn maintainer_problems(
        &self,
        maintainer: &str,
        repository: &str,
    ) -> Result<Vec<Problem>> {
        use tokio_stream::StreamExt;
        let mut all = Vec::new();
        let mut stream = self.maintainer_problems_iter(maintainer, repository);
        while let Some(result) = stream.next().await {
            all.push(result?);
        }
        Ok(all)
    }

    /// Returns a [`Stream`] that automatically paginates through all problems
    /// for a maintainer in a repository.
    ///
    /// Uses [`maintainer_problems_page`](Self::maintainer_problems_page)
    /// under the hood.
    pub fn maintainer_problems_iter<'a>(
        &'a self,
        maintainer: &'a str,
        repository: &'a str,
    ) -> BoxStream<'a, Result<Problem>> {
        Box::pin(async_stream::try_stream! {
            let mut cursor: Option<String> = None;

            loop {
                let page = self.maintainer_problems_page(maintainer, repository, cursor.as_deref()).await?;

                if page.is_empty() {
                    break;
                }

                cursor = page.last().and_then(|p| p.project_name.clone());

                for problem in page {
                    yield problem;
                }

                if cursor.is_none() {
                    break;
                }
            }
        })
    }

    /// Fetch a single page of problems for a maintainer in a repository.
    ///
    /// Pass `None` for the first page, then pass the last `project_name` from
    /// the previous page as the cursor to get the next page.
    pub async fn maintainer_problems_page(
        &self,
        maintainer: &str,
        repository: &str,
        cursor: Option<&str>,
    ) -> Result<Vec<Problem>> {
        let mut url = format!(
            "{}/maintainer/{}/problems-for-repo/{}",
            self.base_url,
            urlencoding(maintainer),
            urlencoding(repository),
        );
        if let Some(start) = cursor {
            url.push_str(&format!("?start={}", urlencoding(start)));
        }
        self.get_json(&url).await
    }
}

fn is_retryable(err: &Error) -> bool {
    match err {
        Error::Http(e) => e.is_connect() || e.is_timeout() || e.is_request(),
        Error::Api { status, .. } => {
            status.is_server_error() || *status == reqwest::StatusCode::TOO_MANY_REQUESTS
        }
        _ => false,
    }
}

fn urlencoding(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projects_url_no_cursor_no_filter() {
        let client = RepologyClient::new().unwrap();
        let filter = ProjectFilter::default();
        let url = client.build_projects_url(None, &filter);
        assert_eq!(url, format!("{}/projects/", DEFAULT_BASE_URL));
    }

    #[test]
    fn projects_url_with_cursor() {
        let client = RepologyClient::new().unwrap();
        let filter = ProjectFilter::default();
        let url = client.build_projects_url(Some("firefox"), &filter);
        assert_eq!(url, format!("{}/projects/firefox/", DEFAULT_BASE_URL));
    }

    #[test]
    fn projects_url_with_filter() {
        let client = RepologyClient::new().unwrap();
        let filter = ProjectFilter::new().inrepo("arch").outdated(true);
        let url = client.build_projects_url(None, &filter);
        assert!(url.contains("inrepo=arch"));
        assert!(url.contains("outdated=1"));
    }

    #[test]
    fn empty_user_agent_rejected() {
        let result = RepologyClient::builder().user_agent("").build();
        assert!(matches!(result, Err(Error::Config(_))));
    }

    #[test]
    fn default_user_agent_works() {
        let client = RepologyClient::new();
        assert!(client.is_ok());
    }
}
