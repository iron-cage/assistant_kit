# Schema: CLR Preferences (`~/.clr/prefs.json`)

### Scope

- **Purpose**: Define the on-disk format of `~/.clr/prefs.json` — the preference file written by `clp .model.select` and read by `clr`.
- **Responsibility**: Authoritative structural reference for `~/.clr/prefs.json`, covering all fields, types, and default behaviors.
- **In Scope**: `subprocess_model` field; file creation on first `clp .model.select id::` write; field read-on-subprocess-launch by `claude_runner_core/src/isolated.rs`; missing-file treated as all-defaults; extra fields tolerated by readers.
- **Out of Scope**: `~/.clr/journal/` directory (→ clr journal); `~/.claude/settings.json` (→ schema/006_settings_json.md); HTTP API payloads; subprocess argument construction (→ docs/subprocess/).

### Fields

| Field | Type | Default | Purpose | Written by | Read by |
|-------|------|---------|---------|------------|---------|
| `subprocess_model` | `string \| null` | *(absent)* | Full model ID for clr run/ask/isolated/refresh subprocesses; absent = use `ISOLATED_DEFAULT_MODEL` | `clp .model.select id::VALUE` | `claude_runner_core/src/isolated.rs` |

**File location:** `~/.clr/prefs.json`

The `~/.clr/` directory is already created by clr for the journal. `prefs.json` is created on first `clp .model.select id::` call. If `~/.clr/` does not exist when `clp .model.select id::` is called, it is created first.

**Extra fields:** Readers (clr) MUST tolerate unknown fields for forward compatibility. Writers (clp) MUST preserve unknown fields when updating.

**Empty file / absent file / null field:** All treated as no preference; clr falls back to `ISOLATED_DEFAULT_MODEL`.

### Example

Pinned state:
```json
{
  "subprocess_model": "claude-opus-4-8"
}
```

After `clp .model.select reset::1`:
```json
{}
```

### Notes

- The `subprocess_model` field accepts any non-empty string. Validity is checked only at clr invocation time — if the model ID is not accepted by the Claude API, the subprocess will error with the API's error response.
- `~/.clr/` is owned by clr (creates it on startup for the journal). `clp .model.select` writes only `prefs.json` within `~/.clr/`.
- Distinct from `~/.claude/settings.json` which governs the interactive Claude Code session model; `prefs.json` governs the clr subprocess model default.

### Features

| File | Relationship |
|------|--------------|
| [feature/069_model_select_command.md](../feature/069_model_select_command.md) | Full feature spec for `.model.select` command |

### Schema

| File | Relationship |
|------|--------------|
| [schema/004_storage_root.md](004_storage_root.md) | `PersistPaths` — `~/.clr/` runtime directory context |
| [schema/006_settings_json.md](006_settings_json.md) | Complementary: interactive session model preference in `settings.json` |
