# claude_code

Behavioral contract tests for the external `claude` binary.

Validates B1–B24 (plus B16h) from `docs/behavior/001_session_behaviors.md`. Tests read
real `~/.claude/` storage and invoke `claude --help` / `--version`. If Claude
Code changes behavior, the corresponding test goes RED.

## Structure

| Path | Responsibility |
|------|----------------|
| `docs/behavior/` | Behavioral documentation for the `claude` binary |
| `src/lib.rs` | Crate documentation |
| `tests/behavior/` | Behavior hypothesis invalidation test suite (B1–B24 + B16h) |

## Running

```bash
cargo nextest run -p claude_code --test behavior
```
