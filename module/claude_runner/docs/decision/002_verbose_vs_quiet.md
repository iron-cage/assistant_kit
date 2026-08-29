# Decision: Verbose vs Quiet

**ID:** D2 · **Category:** Parameter Conventions · **Status:** ✅ Adopted

### Scope

- **Purpose**: Record why runner-internal diagnostics are gated by a boolean `--quiet` rather than a numeric verbosity level.
- **Responsibility**: Rationale for splitting passthrough verbosity, diagnostic suppression, and command preview across three independent flags, and for removing `--verbosity <0-5>`.
- **In Scope**: The `--verbose` / `--quiet` / `--trace` division of labour; which diagnostics `--quiet` suppresses; why the numeric scale was withdrawn (TSK-337).
- **Out of Scope**: Reference semantics of each flag (→ [`../cli/param/`](../cli/param/readme.md)); `--trace` coverage across subprocess-executing commands (→ [`../invariant/004_trace_universality.md`](../invariant/004_trace_universality.md)).

### Decision

Three independent flags, one concern each:

| Flag | Concern | Direction |
|------|---------|-----------|
| `--verbose` | Passthrough — it is a claude-native flag | Handed to the `claude` binary |
| `--quiet` | Runner-internal diagnostics (bool, default `false`) | Suppresses when set |
| `--trace` | Command preview | Independent of both |

When `--quiet` is set, non-fatal CLR diagnostics are suppressed: gate-wait, retry progress, retry-exhaustion, and the keep-claudecode warning. **Fatal errors are always emitted regardless of `--quiet`.** Command preview is handled exclusively by `--trace` and is not tied to any verbosity level.

### Rationale

The former `--verbosity <0-5>` parameter was removed (TSK-337) as an anti-pattern: a 0–5 numeric scale bundles independent output concerns into one opaque integer, violating CLI composability. A user who wants command preview without retry chatter — or the reverse — cannot express that on a single ordinal axis. Three booleans can express every combination; one integer cannot express most of them.

### Consequence

`--verbosity` no longer exists. Passthrough verbosity, diagnostic suppression, and command preview are set independently, and no flag's meaning depends on another's value.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Decision collection index |
| invariant | [`../invariant/004_trace_universality.md`](../invariant/004_trace_universality.md) | `--trace` must be supported by every subprocess-executing command |
| feature | [`../feature/006_cli_design.md`](../feature/006_cli_design.md) | Feature-level view of the CLI design these decisions justify |
