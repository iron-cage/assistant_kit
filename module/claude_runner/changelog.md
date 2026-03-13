# Changelog

All notable changes to claude_runner will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **`--system-prompt <TEXT>` and `--append-system-prompt <TEXT>` flags** (tasks 084-085)
  - `--system-prompt` replaces the default Claude system prompt for the invocation
  - `--append-system-prompt` appends text to the active system prompt (compatible with `--system-prompt`)
  - Both flags forwarded to the `claude` subprocess; CLI delegates to `with_system_prompt()` / `with_append_system_prompt()` builder methods in `claude_runner_core`
  - Documented as Type 6 (`SystemPromptText`), Group 3 (`System Prompt`), params 14–15, Workflow 9
  - `docs/cli/parameter_interactions.md` and `docs/cli/testing/` added to bring crate to L4

### Changed

- **`--dangerously-skip-permissions` is now default-on** (task 058)
  - Every `clr` invocation silently injects `--dangerously-skip-permissions` to avoid stalling automation pipelines
  - New flag `--no-skip-permissions` disables the automatic bypass for contexts requiring human approval gates
  - `--dangerously-skip-permissions` is no longer a user-facing flag; passing it explicitly produces "unknown option" error
  - Default Flags Principle documented in `spec.md § Default Flags Principle`

- **Process management moved to `claude_runner_core`** (task 037)
  - `ClaudeCommand` builder and `execute()` moved from `claude_runner` to `claude_runner_core`
  - `claude_runner` binary now delegates process execution to `claude_runner_core`
  - Library surface intentionally minimal: only `COMMANDS_YAML` path and `VerbosityLevel`
  - `stale_ref_guard_test.rs` guards against any regression to pre-migration import paths

- **`architecture_migration_plan.md` removed** (task 037)
  - Post-migration artifact; no longer needed now that the move is complete

### Documentation

- **CLI documentation synced with unilang 5-phase pipeline migration** (task 040)
  - `docs/cli/commands.md`, `params.md`, `types.md`, `dictionary.md`,
    `parameter_groups.md`, `workflows.md` updated to reflect 5-phase architecture
  - `docs/design_decisions.md` updated with post-migration rationale
