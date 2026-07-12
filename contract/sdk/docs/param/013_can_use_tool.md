# canUseTool

Per-tool-call permission override callback.

### Forms

| | Value |
|-|-------|
| TS Field | `canUseTool?: CanUseTool` |
| Python Field | `can_use_tool` (inferred snake_case-identical) |
| CLI Equivalent | — (N/A; no per-call callback mechanism exists at the CLI level) |

### Type

function — see [`../api/006_permission_callback.md`](../api/006_permission_callback.md) for the full `CanUseTool` signature

### Default

`undefined`

### Since

SDK GA

### Description

The only field in this curated set with genuinely no CLI counterpart at all — the CLI's permission surface (`--permission-mode`, `--allowed-tools`, `--disallowed-tools`, `--dangerously-skip-permissions`) is entirely static/argv-time, while `canUseTool` is a live callback invoked once per tool-use request, with access to the specific `toolName`, `input`, and (per [S6](../behavior/006_s6_permission_modes_richer_than_cli.md)/[`../api/006_permission_callback.md`](../api/006_permission_callback.md)) an `agentID` distinguishing subagent-originated calls. This is the field a Rust bridge would most need a protocol-level equivalent for if it wants dynamic, request-time permission logic rather than the static allow/deny lists a CLI-flag-based approach is limited to.

### Cross-References

| Type | File | Responsibility |
|------|------|-----------------|
| doc | [readme.md](readme.md) | Master curated parameter table |
| doc | [003_permission_mode.md](003_permission_mode.md) | Coarser session-wide setting this overrides per call |
| doc | [../api/006_permission_callback.md](../api/006_permission_callback.md) | Full callback signature |
