# Changelog

All notable changes to **MacroMag OS** (`MacroMag-32`) are documented in this file (standard [Keep a Changelog](https://keepachangelog.com) and [Semantic Versioning](https://semver.org)).

## [Unreleased]
- Planned updates include Rust `ServiceManager` (#29), GPT/UEFI boot support (#31), persistent root filesystem (#32), `zram`/`/swapfile` creation (#33), multi-user login (#34), package dependency resolution (#37), network stack improvements (#39), and interactive shell features (#40).

## [0.3.0-mvp] - 2026-11-23
### Added
- VMware setup guides, Bare-Metal Rootfs Installer, native Rust VOS hardware/system built-ins, Git-Free MUR Package Manager, and automated CI/CD GitHub Actions workflow.
### Changed
- Separated strongly-typed VOS pipelines from POSIX pipes and redirections.
### Fixed
- PID 1 Init zombie process reaping, `/dev/pts` mounting, and background logging.

## [0.2.0-alpha] - 2026-08-15
### Added
- MUR declarative manifest format and static `x86_64-unknown-linux-musl` target integration.

## [0.1.0-prototype] - 2026-03-01
### Added
- Initial VOS syntax design, lexer/parser prototypes, and foundational layout.
