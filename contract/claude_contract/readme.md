# claude_contract

Behavioral contract tests for the external `claude` binary.

Validates B1–B18 from `docs/claude_code/001_session_behaviors.md`. Tests read
real `~/.claude/` storage and invoke `claude --help` / `--version`. If Claude
Code changes behavior, the corresponding test goes RED.

## Structure

| Path | Responsibility |
|------|----------------|
| `src/lib.rs` | Crate documentation |
| `tests/behavior/` | Behavior hypothesis invalidation test suite (B1–B18) |

## Running

```bash
cargo nextest run -p claude_contract --test behavior
```
