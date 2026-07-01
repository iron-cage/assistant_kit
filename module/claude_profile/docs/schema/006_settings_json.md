# Schema: Session Settings — `~/.claude/settings.json`

### Scope

- **Purpose**: Define which fields in `~/.claude/settings.json` are read or written by `clp`, their semantics, and the write callers.
- **Responsibility**: Documents the `settings.json` fields that `clp` reads or writes and their write rules.
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
| `effortLevel` | string or absent | absent → initialized by `apply_model_override()` on first use | Effort level for interactive sessions (`"low"`, `"normal"`, `"high"`, `"max"`). Controls extended thinking depth. | `apply_model_override()` writes unconditionally: `"max"` for Opus, `"high"` for Sonnet/absent-tier (TSK-335); BUG-312 fallback `"high"` retained as unreachable safety net | `get_session_effort()` — footer `Current` line in `.usage`; `Next` line uses model-derived effort from `recommended_model()` |

### Write Rules

- `clp` reads the entire `settings.json` into memory, modifies only `model` or `effortLevel`, and writes it back via `json_serialize_flat_object` — all other fields are preserved.
- Never `serde_json::to_string` — the hand-rolled formatter already produces pretty output.

### Effort Tracking Behavior (Fix BUG-312, Fix BUG-322, TSK-335)

`apply_model_override()` writes `effortLevel` unconditionally on every call regardless of whether the model changed: `"max"` for Opus branch, `"high"` for Sonnet and absent-tier branches (TSK-335). The BUG-312 fallback guard (`get_session_effort().is_none()` → `"high"`) is retained as unreachable safety net. The rotation carry-forward `set_session_effort()` was removed — `apply_model_override()` owns all effort writes.

### Features

| File | Relationship |
|------|-------------|
| [feature/034_explicit_session_model_override.md](../feature/034_explicit_session_model_override.md) | `set_session_model()` and `get_session_model()` |
| [feature/035_model_command.md](../feature/035_model_command.md) | `.model` command; `map_model_shorthand()` |
| [feature/062_unified_session_config.md](../feature/062_unified_session_config.md) | `set_session_effort()`, footer effort display |

### Schema

| File | Relationship |
|------|-------------|
| [003_file_topology.md](003_file_topology.md) | `settings_file()` path method |

### Invariants

| File | Relationship |
|------|-------------|
| [invariant/007](../invariant/007_json_storage_format.md) | Exception: `json_serialize_flat_object` is exempt |
