# Behavior B8: Zero-Byte Session Placeholder on Startup

### Scope

- **Purpose**: Document that Claude Code creates an empty `.jsonl` placeholder on startup that persists if the process exits before writing any entries.
- **Responsibility**: Authoritative instance for behavior B8 — defines the behavior statement, certainty level, and supporting evidence. Tier is UNVERIFIED (no hard assertion in test).
- **In Scope**: Zero-byte file creation at startup; persistence on crash/early exit; observed presence in storage.
- **Out of Scope**: Normal session file growth after entries are written (→ [B2](002_b2_new_session_creates_file.md)).

### Behavior

**Status**: 🎯 Observed | **Certainty**: 85% | **Tier**: UNVERIFIED | **Since**: pre-v1.0 | **Evidence**: E7, E18

Claude Code creates an empty `.jsonl` file as a session placeholder at startup. If the process crashes or exits before writing any entries, the file remains at 0 bytes.

Zero-byte `.jsonl` files have been observed in real `~/.claude/` storage alongside non-empty session files. This is consistent with a pattern where the file is pre-created to reserve the session identity, and entries are appended only as the conversation proceeds.

The UNVERIFIED tier means the test logs this observation but makes no hard assertion — no `assert!` that would cause a RED failure if the behavior changes.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E7 | B8 | Observation | Live storage | `~/.claude/projects/*/` | Zero-byte `.jsonl` files observed in project directories alongside non-empty sessions |
| E18 | B8 | Observation | `../../tests/behavior/b08_zero_byte_init.rs` | `b8_zero_byte_jsonl_exists_in_real_storage` | Zero-byte `.jsonl` files observed in real `~/.claude/` storage (test logs observation, does not assert) |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [002_b2_new_session_creates_file.md](002_b2_new_session_creates_file.md) | Normal file creation per invocation |
| test | `../../tests/behavior/b08_zero_byte_init.rs` | Observation test (UNVERIFIED — no hard assertion) |
