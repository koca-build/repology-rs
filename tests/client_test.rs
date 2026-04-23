use std::sync::OnceLock;

use repology::{PackageStatus, ProblemType, ProjectFilter, RepologyBlockingClient, RepologyClient};

fn client() -> &'static RepologyBlockingClient {
    static CLIENT: OnceLock<RepologyBlockingClient> = OnceLock::new();
    CLIENT.get_or_init(|| RepologyBlockingClient::new().unwrap())
}

#[test]
fn project_firefox() {
    let packages = client().project("firefox").unwrap();

    assert!(!packages.is_empty(), "firefox should have packages");
    assert!(packages.iter().any(|p| p.repo == "arch"));
    assert!(packages.iter().all(|p| !p.repo.is_empty()));
    assert!(packages.iter().all(|p| !p.version.is_empty()));

    assert!(
        packages
            .iter()
            .any(|p| p.status == Some(PackageStatus::Newest))
    );
}

#[test]
fn project_nonexistent() {
    let packages = client()
        .project("this_project_surely_does_not_exist_xyz_999")
        .unwrap();

    assert!(packages.is_empty());
}

#[test]
fn projects_filtered() {
    let filter = ProjectFilter::new()
        .inrepo("debian_12")
        .outdated(true)
        .search("python");

    let projects = client().projects_page(&filter, None).unwrap();

    assert!(
        !projects.is_empty(),
        "should find outdated python packages in debian_12"
    );
    for (_name, packages) in &projects {
        assert!(!packages.is_empty());
        assert!(
            packages
                .iter()
                .any(|p| p.status == Some(PackageStatus::Outdated))
        );
    }
}

#[test]
fn projects_pagination() {
    let filter = ProjectFilter::new().inrepo("debian_12").outdated(true);

    let page1 = client().projects_page(&filter, None).unwrap();
    assert!(!page1.is_empty());

    let cursor = page1.keys().max().unwrap();
    let page2 = client().projects_page(&filter, Some(cursor)).unwrap();
    assert!(!page2.is_empty());

    assert!(page2.contains_key(cursor));

    let new_keys: Vec<_> = page2
        .keys()
        .filter(|k| !page1.contains_key(k.as_str()))
        .collect();
    assert!(!new_keys.is_empty(), "second page should have new projects");
}

#[test]
fn projects_auto_paginate() {
    let filter = ProjectFilter::new()
        .inrepo("debian_12")
        .outdated(true)
        .search("lib");

    let all = client().projects(&filter).unwrap();
    let page = client().projects_page(&filter, None).unwrap();

    assert!(all.len() >= page.len());
}

#[test]
fn repository_problems_freebsd() {
    let problems = client().repository_problems_page("freebsd", None).unwrap();

    assert!(!problems.is_empty());
    assert!(
        problems
            .iter()
            .any(|p| p.problem_type == ProblemType::HomepageDead)
    );
    assert!(problems.iter().all(|p| p.project_name.is_some()));
}

#[test]
fn repository_problems_nonexistent() {
    let result = client().repository_problems_page("this_repo_does_not_exist_xyz", None);

    assert!(
        matches!(result, Err(repology::Error::Api { .. })),
        "nonexistent repo should return API error"
    );
}

#[test]
fn maintainer_problems() {
    let problems = client()
        .maintainer_problems_page("ports@freebsd.org", "freebsd", None)
        .unwrap();

    assert!(!problems.is_empty());
    assert!(
        problems
            .iter()
            .all(|p| p.maintainer.as_deref() == Some("ports@freebsd.org"))
    );
}

#[test]
fn vulnerable_field_present() {
    let packages = client().project("openssl").unwrap();

    assert!(
        packages.iter().any(|p| p.vulnerable),
        "openssl should have vulnerable packages"
    );
}

#[test]
fn package_has_binnames_from_single_endpoint() {
    let packages = client().project("firefox").unwrap();

    assert!(
        packages.iter().any(|p| p.binnames.is_some()),
        "single-project endpoint should return binnames"
    );
}

#[test]
fn builder_custom_user_agent() {
    let client = RepologyBlockingClient::builder()
        .user_agent("repology-rs-test/0.1")
        .build()
        .unwrap();

    let packages = client.project("vim").unwrap();
    assert!(!packages.is_empty());
}

#[test]
fn new_returns_working_client() {
    let client = RepologyClient::new().unwrap();
    assert!(std::mem::size_of_val(&client) > 0);
}

#[test]
fn empty_user_agent_rejected() {
    let result = RepologyClient::builder().user_agent("").build();
    assert!(matches!(result, Err(repology::Error::Config(_))));
}
