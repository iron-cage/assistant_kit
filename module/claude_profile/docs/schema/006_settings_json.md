# Schema: Session Settings — `~/.claude/settings.json`

### Scope

- **Purpose**: Define which fields in `~/.claude/settings.json` are read or written by `clp`, their semantics, and the write callers.
- **In Scope**: All `settings.json` fields that `clp` touches — `model` and `effortLevel`. All other fields are owned by the Claude binary and must never be modified.
- **Out of Scope**: Full `settings.json` schema (not owned by clp); CLI rendering of these values.

### File Location

```
~/.claude/settings.json
```

Path via `ClaudePaths::settings_file()`. See [schema/003](003_file_topology.md).

### Format

Single-level JSON object (hand-rolled formatter in `settings_io.rs`, not `serde_json::to_string_pretty`). The `json_serialize_flat_object` formatter is exempt from the invariant/007 `to_string_pretty` rule because it already produces equivalent pretty output.

### Fields Managed by `clp`

| Field | Type | Default | Semantics | Written by | Read by |
|-------|------|---------|-----------|-----------|---------|
| `model` | string or absent | absent | Session model shorthand (`"sonnet"`, `"opus"`, `"haiku"`, or full model ID). Controls which Claude model is used for interactive sessions. | `set_session_model()`, `switch_account()` (restores from `{name}.json`), `.model set::`, `.account.use set_model::`, `apply_model_override()` (Fix BUG-311: bidirectional), `set_session_effort()` init path | `get_session_model()`, `.usage`/`.accounts` `model::1`, `recommended_model()` in `format.rs` |
| `effortLevel` | string or absent | absent → initialized to `"low"` by `apply_model_override()` on first use (Fix BUG-312) | Effort level for interactive sessions (`"low"`, `"normal"`, `"high"`, `"max"`). Controls extended thinking depth. | `set_session_effort()` called during rotation carry-forward; `apply_model_override()` initializes to `"low"` when absent (Fix BUG-312) | `get_session_effort()`, footer `Next` line in `.usage` |

### Write Rules

- `clp` reads the entire `settings.json` into memory, modifies only `model` or `effortLevel`, and writes it back via `json_serialize_flat_object` — all other fields are preserved.
- Never `serde_json::to_string` — the hand-rolled formatter already produces pretty output.

### Initialization Behavior (Fix BUG-312)

When `effortLevel` is absent from `settings.json`, `apply_model_override()` writes `"low"` on first use. This prevents the `.usage` footer from showing no effort level after initial rotation.

### Cross-References

| File | Relationship |
|------|-------------|
| [feature/034_explicit_session_model_override.md](../feature/034_explicit_session_model_override.md) | `set_session_model()` and `get_session_model()` |
| [feature/035_model_command.md](../feature/035_model_command.md) | `.model` command; `map_model_shorthand()` |
| [feature/062_unified_session_config.md](../feature/062_unified_session_config.md) | `set_session_effort()`, footer effort display |
| [schema/003](003_file_topology.md) | `settings_file()` path method |
| [invariant/007](../invariant/007_json_storage_format.md) | Exception: `json_serialize_flat_object` is exempt |
