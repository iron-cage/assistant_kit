//! Behavioral contract tests for the external `claude` binary.
//!
//! Validates B1–B18 from `docs/behavior/001_session_behaviors.md`.
//! Tests read real `~/.claude/` storage and invoke `claude --help` / `--version`.
//!
//! These are not unit tests of any workspace crate. They verify that the
//! external `claude` binary upholds the behavioral contract this workspace
//! depends on. When Claude Code changes behavior, the corresponding test
//! goes RED.
