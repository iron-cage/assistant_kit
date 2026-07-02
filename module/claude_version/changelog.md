# Changelog

All notable changes to claude_version will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **`.config` command** — unified 4-layer configuration management; replaces `.settings.show`, `.settings.get`, `.settings.set`
  - Modes: show-all / get / set / unset; `scope::` (`user` or `project`), `unset::1`, `dry::` preview
  - 4-layer resolution: env → project → user → catalog default; see `algorithm/002_config_resolution.md`

- **`.params` command** — read-only Claude Code parameter catalog inspector
  - Shows all params with CLI flag, env var, config key forms and current observable values
  - `key::K` deep-dive mode; `kind::config` / `kind::env` filter; `format::json` output

### Changed

- **Binary renamed `clm` → `claude_version`; `clv` alias added**
  - Canonical invocation: `claude_version .status`, etc.
  - Short alias `clv` also installed (both built from the same `src/main.rs`)
  - All integration tests updated: `CARGO_BIN_EXE_clm` → `CARGO_BIN_EXE_claude_version`
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

- **`verbosity::` parameter now range-validated** (issue-verbosity-bypass)
  - Only `v::` alias was guarded; `verbosity::3` exited 0, `verbosity::-1` defaulted to 1
  - Both forms now reject values outside 0–2 range

- **`count::`/`interval::` overflow produces clear error** (issue-count-overflow)
  - Values > i64::MAX triggered opaque overflow error in unilang
  - Now validates as non-negative integer within i64 range before dispatch

- **Idempotent install now persists preferred version** (issue-358)
  - Early return on "already at target" bypassed `store_preferred_version()`
  - Every exit path confirming version now persists spec + resolved fields

- **Process kill reports signal delivery errors** (issue-kill-silent)
  - `let _ = kill()` silently discarded errors; EPERM was invisible
  - Errors now collected and reported; benign ESRCH filtered from final report

- **`.settings.set` rejects empty value** (issue-settings-set-empty-value)
  - Used lenient `require_string_arg` (allows empty) instead of `require_nonempty_string_arg`
  - `value::` with empty string now exits 1 with error message

### Changed

- **Binary renamed `claude_version` → `cm`** (task 036)
  - Shorter invocation: `cm .status`, `cm .account.list`, etc.
  - `cm .` now shows help (previously triggered an error)
  - All integration tests updated to use `env!("CARGO_BIN_EXE_cm")`

- **Account management delegated to `claude_profile`** (task 038)
  - `claude_version` account commands (`.account.list`, `.account.status`,
    `.account.switch`) now call `claude_profile` library functions
  - Account storage logic removed from `claude_version` — single owner: `claude_profile`

### Added

- **`.version::` parameter for `.version.guard`** (task 039)
  - Guards against unmatched Claude Code version at startup
  - `cm .version.guard version::1.0.0` exits non-zero if installed version differs
  - Replaces hard-coded version expectation with a configurable parameter

### Documentation

- **CLI documentation synced with unilang 5-phase pipeline migration** (task 040)
  - `docs/cli/001_commands.md`, `005_params.md`, `006_types.md`, `002_dictionary.md`,
    `003_parameter_groups.md`, `007_workflows.md` updated to reflect 5-phase architecture
  - Testing documentation files added under `tests/docs/cli/`
