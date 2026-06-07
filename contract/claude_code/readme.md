# claude_code

Behavioral contract tests for the external `claude` binary.

Validates B1–B24 (plus B16h) from `docs/behavior/readme.md`. Tests read
real `~/.claude/` storage and invoke `claude --help` / `--version`. If Claude
Code changes behavior, the corresponding test goes RED.

## Structure

| Path | Responsibility |
|------|----------------|
| `docs/` | Claude Code contract specifications (10 entity types, 137 instances) |
| `src/lib.rs` | Crate documentation |
| `tests/behavior/` | Behavior hypothesis invalidation test suite (B1–B24 + B16h) |
| `tests/docs/` | Test spec documents for fault classification (FT-01–FT-07) |

## Running

```bash
cargo nextest run -p claude_code --test behavior
```
