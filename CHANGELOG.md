# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.2](https://github.com/koca-build/repology-rs/compare/v0.1.1...v0.1.2) - 2026-04-23

### Added

- add exponential backoff retries ([#3](https://github.com/koca-build/repology-rs/pull/3))

## [0.1.1](https://github.com/koca-build/repology-rs/compare/v0.1.0...v0.1.1) - 2026-04-23

### Fixed

- use `values()` instead of destructuring in map iteration

### Other

- return BoxStream from *_iter methods to remove pin requirement ([#2](https://github.com/koca-build/repology-rs/pull/2))
- add CI, PR linting, and release-plz workflows
