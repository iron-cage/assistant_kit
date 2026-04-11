# Changelog

All notable changes to claude_runner will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- **Empty positional arg `""` after `--` separator no longer produces degenerate `"ultrathink "` message** (issue-empty-msg-double-dash)
  - `clr -- ""` now behaves identically to `clr --` (no message, no `--print`, interactive REPL)
  - Root cause: the `--` arm in `parse_args` used `positional.extend()` which copies all tokens verbatim; the empty-token guard in the `_` arm did not apply to this code path
  - Fix: filter empty tokens in the `--` arm via `.filter(|t| !t.is_empty())` before extending positional
  - Reproducer: `t57_empty_positional_after_double_dash_ignored` in `tests/cli_args_test.rs`

- **Empty positional arg `""` no longer produces degenerate `"ultrathink "` message** (issue-empty-msg-ultrathink)
  - `clr ""` now behaves identically to bare `clr` (no message, no `--print`, interactive REPL)
  - Root cause: empty token was pushed to positional list, joined to `message = Some("")`, then the ultrathink prefix produced `"ultrathink "` (trailing space) and triggered print mode
  - Fix: skip empty tokens in the positional-arg collection path of `parse_args`
  - Reproducer: `t54_empty_positional_arg_ignored` in `tests/cli_args_test.rs`

- **`--help`/`-h` now wins over unknown flags regardless of position** (issue-help-loses-to-unknown)
  - `clr --help --unknown` and `clr --unknown --help` both now exit 0 and show USAGE
  - Root cause: `parse_args` returned `Err` immediately on the first unknown flag; `main()` then took the error path before ever consulting `cli.help`
  - Fix: pre-scan for `--help`/`-h` at the top of `parse_args`; if found, return `CliArgs { help: true, .. }` immediately without full parsing
  - Reproducers: `t55_help_wins_over_subsequent_unknown_flag` and `t56_help_wins_over_preceding_unknown_flag` in `tests/cli_args_test.rs`

### Added

- **Default `"ultrathink "` message prefix with `--no-ultrathink` opt-out** (task 090)
  - Every `clr` invocation prepends `"ultrathink "` to the message before forwarding to the `claude` subprocess, activating extended thinking mode for all automation without user intervention
  - `--no-ultrathink` flag disables the automatic prefix for callers managing their own prompt structure
  - Idempotent guard: messages already beginning with `"ultrathink"` are not double-prefixed
  - Single injection site in `build_claude_command()` via `effective_msg` local; all paths share the same transformation
  - Documented in `docs/invariant/001_default_flags.md` as the fourth default injection alongside `-c`, `--dangerously-skip-permissions`, and `--chrome`
  - 7 new tests (T50-T53 + 3 dry-run tests); 9 stale test assertions updated to reflect prefixed output

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
