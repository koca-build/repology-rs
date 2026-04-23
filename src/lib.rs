//! # repology
//!
//! Rust client for the [Repology API](https://repology.org/api/v1).
//!
//! ## Quick Start
//!
//! ```no_run
//! use repology::{RepologyClient, ProjectFilter};
//!
//! # async fn example() -> repology::Result<()> {
//! let client = RepologyClient::new()?;
//!
//! // Fetch a single project
//! let packages = client.project("firefox").await?;
//! println!("firefox has {} packages", packages.len());
//!
//! // Find outdated packages in Debian 12 (Bookworm)
//! let filter = ProjectFilter::new()
//!     .inrepo("debian_12")
//!     .outdated(true);
//! let projects = client.projects(&filter).await?;
//! for (name, packages) in &projects {
//!     println!("{name}: {}", packages[0].version);
//! }
//! # Ok(())
//! # }
//! ```

pub mod blocking;
mod client;
mod error;
mod filter;
pub mod models;

pub use blocking::RepologyBlockingClient;
pub use client::{RepologyClient, RetryConfig};
pub use error::{Error, Result};
pub use filter::ProjectFilter;
pub use models::{Package, PackageStatus, Problem, ProblemType};
