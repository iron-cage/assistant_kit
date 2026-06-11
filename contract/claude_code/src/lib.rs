//! Behavioral contract tests for the external `claude` binary.
//!
//! Validates B1–B24 (plus B16h) from `docs/behavior/readme.md`.
//! Tests read real `~/.claude/` storage and invoke `claude --help` / `--version`.
//!
//! These are not unit tests of any workspace crate. They verify that the
//! external `claude` binary upholds the behavioral contract this project
//! depends on. When Claude Code changes behavior, the corresponding test
//! goes RED.
