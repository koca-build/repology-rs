# repology

[![Crates.io](https://img.shields.io/crates/v/repology.svg)](https://crates.io/crates/repology)
[![Documentation](https://docs.rs/repology/badge.svg)](https://docs.rs/repology)
[![License: MIT](https://img.shields.io/crates/l/repology.svg)](LICENSE)

Rust client for the [Repology API](https://repology.org/api/v1).

Provides async and blocking interfaces with built-in rate limiting, auto-pagination, and strongly-typed models.

## Usage

```rust
use repology::{RepologyClient, ProjectFilter};

#[tokio::main]
async fn main() -> repology::Result<()> {
    let client = RepologyClient::new()?;

    // Fetch a single project
    let packages = client.project("firefox").await?;
    println!("firefox is packaged in {} repositories", packages.len());

    // Find outdated packages in Debian 12
    let filter = ProjectFilter::new()
        .inrepo("debian_12")
        .outdated(true);
    let projects = client.projects(&filter).await?;
    for (name, packages) in &projects {
        println!("{name}: {}", packages[0].version);
    }

    Ok(())
}
```

### Blocking

```rust
use repology::RepologyBlockingClient;

let client = RepologyBlockingClient::new()?;
let packages = client.project("firefox")?;
```

### Pagination

Methods like `projects()` and `repository_problems()` auto-paginate through all results. For manual control or lazy iteration:

```rust
// Lazy streaming (async)
use tokio_stream::StreamExt;

let mut stream = std::pin::pin!(client.projects_iter(&filter));
while let Some(result) = stream.next().await {
    let (name, packages) = result?;
    println!("{name}");
}

// Manual page-by-page
let page1 = client.projects_page(&filter, None).await?;
let cursor = page1.keys().max().unwrap();
let page2 = client.projects_page(&filter, Some(cursor)).await?;
```

## Rate Limiting

The Repology API requires bulk clients to make no more than 1 request per second. This is enforced automatically by default. Configure via the builder:

```rust
use std::time::Duration;

let client = RepologyClient::builder()
    .user_agent("my-app/1.0 (https://github.com/me/my-app)")
    .rate_limit(Duration::from_secs(2))
    .build()?;
```

## License

MIT
