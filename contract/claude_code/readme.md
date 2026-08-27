# claude_code

Behavioral contract tests for the external `claude` binary.

Validates B1–B26, B16h, and B37 from `docs/behavior/readme.md` — 28 test files.
Tests read real `~/.claude/` storage and invoke `claude --help` / `--version`.
If Claude Code changes behavior, the corresponding test goes RED.

**Coverage gap:** B27–B36 have no test file. Those ten behaviors rest on one-off
experiments and binary analysis; nothing goes RED if they regress. See the
Invalidation Tests table in `docs/behavior/readme.md`, which lists each absent
file alongside what it would assert.

**Tier caveat:** a passing test does not imply the behavior is confirmed. The
`NEG-ONLY` tier in particular passes identically for an env var that does not
exist in the binary at all — B11 and B23 were both refuted on that basis while
their tests stayed green. Read the tier column before trusting a green suite.

## Structure

| Path | Responsibility |
|------|----------------|
| `docs/` | Claude Code contract specifications (16 collections, 434 instances) |
| `src/lib.rs` | Crate documentation |
| `tests/behavior/` | Behavior invalidation test suite (B1–B26, B16h, B37) |
| `tests/docs/` | Test spec documents for fault classification |

## Running

```bash
cd contract/claude_code && cargo nextest run --test behavior
```
