//! Behavioral contract tests for the external `claude` binary.
//!
//! Validates B1–B26, B16h, and B37 from `docs/behavior/readme.md` — 28 test files.
//! Tests read real `~/.claude/` storage and invoke `claude --help` / `--version`.
//!
//! These are not unit tests of any workspace crate. They verify that the
//! external `claude` binary upholds the behavioral contract this project
//! depends on. When Claude Code changes behavior, the corresponding test
//! goes RED.
//!
//! Two caveats on reading a green run, both detailed in `docs/behavior/readme.md`:
//!
//! - **B27–B36 have no test file.** Nothing goes RED if those ten behaviors
//!   regress; they rest on one-off experiments and binary analysis.
//! - **A pass does not imply the behavior is confirmed.** The `NEG-ONLY` tier
//!   asserts only that the binary does not *reject* something — which holds
//!   identically for an env var that is honored, silently ignored, or absent
//!   from the binary entirely. B11 and B23 were both refuted on that basis
//!   while their tests stayed green. Check the tier column before trusting a
//!   green suite.
