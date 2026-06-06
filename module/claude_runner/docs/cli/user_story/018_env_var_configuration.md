# CLI User Story: Env-var Configuration

### Scope

- **Purpose**: Document the CLR_* environment variable system as a configuration layer.
- **Responsibility**: Define acceptance criteria for env var fallback behavior, CLI precedence, bool semantics, and dry-run discoverability.
- **In Scope**: CLR_* fallback application, CLI-wins rule, bool var semantics (1/true only), dry-run effective value display.
- **Out of Scope**: Individual parameter semantics (→ param/ docs), subprocess env vars (→ CLAUDE_CODE_MAX_OUTPUT_TOKENS).

### Persona

Developer or CI system that configures `clr` defaults via environment variables for scripted or automated invocations, without modifying every command.

### Goal

Set `clr` defaults via CLR_* environment variables so that repeated invocations share configuration without specifying flags on every call, while allowing per-invocation overrides via explicit CLI flags.

### Acceptance Criteria

- A CLR_* env var applies when the corresponding CLI flag is absent (fallback behavior)
- An explicit CLI flag always wins over the corresponding CLR_* env var (CLI precedence)
- Bool CLR_* vars accept `"1"` or `"true"` (case-insensitive) as truthy; all other values (including `"yes"`, `"0"`, `"false"`, empty, or absent) are treated as false
- Invalid CLR_* values for parsed types (int, enum) are silently ignored (field stays at default)
- `--dry-run` reveals the effective values after env var application

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`run`](../command/01_run.md) | All 30 CLR_* vars apply to `run` |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Claude-Native Flags](../param_group/01_claude_native_flags.md) | Several members have CLR_* env var counterparts |
| 2 | [Runner Control](../param_group/02_runner_control.md) | Several members have CLR_* env var counterparts |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 3 | [`--model`](../param/003_model.md) | `CLR_MODEL` — example env var for string type |
| 11 | [`--dry-run`](../param/011_dry_run.md) | Reveals effective env-var values without executing |
| 12 | [`--verbosity`](../param/012_verbosity.md) | `CLR_VERBOSITY` — example env var for parsed type |

### Related User Stories

| # | User Story | Relationship |
|---|------------|--------------|
| 4 | [Dry-run Preview](004_dry_run_preview.md) | `--dry-run` is the discoverability mechanism for env var values |
| 17 | [Model Selection](017_model_selection.md) | `CLR_MODEL` is an instance of the CLR_* env var system |
