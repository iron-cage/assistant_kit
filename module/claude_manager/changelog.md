# Changelog

All notable changes to claude_manager will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- **Binary renamed `clm` → `claude_manager`; `clman` alias added**
  - Canonical invocation: `claude_manager .status`, etc.
  - Short alias `clman` also installed (both built from the same `src/main.rs`)
  - All integration tests updated: `CARGO_BIN_EXE_clm` → `CARGO_BIN_EXE_claude_manager`
  - Spec: Architecture table binary name entry updated

### Fixed

- **`.version.guard` watch loop no longer exits on install error** (task 046)
  - Transient failures (e.g. `ETXTBSY` "Text file busy" when a running session
    holds the binary file descriptor) were propagating out of the watch loop and
    terminating the daemon after a single failed restore attempt
  - Errors are now logged to stderr (`[HH:MM:SS] #N error: ...`) and the loop
    continues to the next interval — the guard keeps watching after any error
  - One-shot mode (`interval::0`) behavior is unchanged: errors still exit non-zero
  - Spec: FR-19 — TC-415 added to regression suite

### Changed

- **Binary renamed `claude_manager` → `cm`** (task 036)
  - Shorter invocation: `cm .status`, `cm .account.list`, etc.
  - `cm .` now shows help (previously triggered an error)
  - All integration tests updated to use `env!("CARGO_BIN_EXE_cm")`

- **Account management delegated to `claude_profile`** (task 038)
  - `claude_manager` account commands (`.account.list`, `.account.status`,
    `.account.switch`) now call `claude_profile` library functions
  - Account storage logic removed from `claude_manager` — single owner: `claude_profile`

### Added

- **`.version::` parameter for `.version.guard`** (task 039)
  - Guards against unmatched Claude Code version at startup
  - `cm .version.guard version::1.0.0` exits non-zero if installed version differs
  - Replaces hard-coded version expectation with a configurable parameter

### Documentation

- **CLI documentation synced with unilang 5-phase pipeline migration** (task 040)
  - `docs/cli/commands.md`, `params.md`, `types.md`, `dictionary.md`,
    `parameter_groups.md`, `workflows.md` updated to reflect 5-phase architecture
  - Testing documentation files added under `docs/cli/testing/`
