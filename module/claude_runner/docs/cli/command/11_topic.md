# CLI Command: topic

### Description

Creates or continues a named workspace session: a hyphenated subdirectory of the current working directory with its own isolated Claude Code conversation history. Reuses `run`'s entire parameter set and execution path — the only behavioral difference is `--subdir`'s default.

-- **Parameters:** all parameters from `run` (identical defaults, except `--subdir` — see below)
-- **Exit Codes:** 0 (success) | 1 (error) | 2 (rate-limit/transient) | 3 (expect mismatch) | 4 (timeout) | N (subprocess passthrough) | 128+signal (signal)

### Syntax

```sh
clr topic [OPTIONS] [MESSAGE]
```

### Parameters

All parameters from [`run`](01_run.md) are accepted with identical defaults, with one exception:

| Parameter | `run`/`ask` default | `topic` default |
|-----------|----------------------|------------------|
| `--subdir` | `.` (identity — no subdirectory) | auto-generated slug from `MESSAGE` when omitted; explicit `--subdir NAME` overrides |

**Algorithm (2 steps):**
1. If `--subdir` is explicitly given (any non-identity value), skip to step 2 with that value unchanged — `topic` behaves exactly like `ask` from here.
2. Otherwise, derive a slug from `MESSAGE` (lowercase; first few words; non-alphanumeric runs collapsed to a single `-`; truncated to a concise length) and disambiguate it against existing subdirectories of the effective `--dir` by appending `-2`, `-3`, ... until the candidate name does not already exist on disk. Use the disambiguated slug as `--subdir`'s value, then delegate to `run`'s execution path unchanged.

### Execution Modes

| Invocation | Mode | Path |
|------------|------|------|
| `clr topic "Investigate the flaky test"` | Print (default) | slug generated, then `execute()` + `--print` |
| `clr topic --subdir auth-refactor "Continue the auth work"` | Print (default) | explicit name, then `execute()` + `--print` |
| `clr topic --dry-run "Investigate the flaky test"` | Preview only | slug generated, then `describe()` / `describe_env()` |
| `clr topic --interactive --subdir auth-refactor` | Interactive | explicit name, then `execute_interactive()` |

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Error (parse failure, execution error) |
| 2 | Rate-limit passthrough or Transient retries exhausted |
| 3 | Expect mismatch — output did not match `--expect` values after all retries |
| 4 | CLR-layer watchdog timeout: subprocess exceeded `--timeout`; stderr contains "Error: timeout after Ns" |
| N | Passthrough from claude subprocess |
| 128+signal | Subprocess killed by signal; follows POSIX convention (e.g., SIGTERM → 143, SIGKILL → 137) |

### Examples

```sh
# Auto-named topic: slug generated from the message
clr topic "Investigate the flaky concurrency-gate test"
# -> effective dir ends with /-investigate-the-flaky (example slug)

# Explicit topic name — first call clones the current session into it
clr topic --subdir auth-refactor "Start refactoring the auth module"

# Same explicit name — second call continues that topic's own conversation
clr topic --subdir auth-refactor "What did we change so far?"

# Auto-naming with a collision: counter suffix disambiguates
clr topic "Investigate the flaky concurrency-gate test"
# -> a second, independent topic: /-investigate-the-flaky-2

# Cross-load into a topic from a different source project
clr topic --subdir shared-fix --from ~/project-a "Port this fix"
```

### Notes

`topic` is a pre-configured alias of `run`/`ask` — it changes only `--subdir`'s default value, per the [Representation Absorption Test](../command_group/01_run_ask.md#representation-absorption-test); no new dispatch logic beyond the slug-generation step exists.

**Clone vs. continue:** whether a given topic name clones a fresh session or continues an existing one is determined entirely by the pre-existing `--subdir` + `--from` session-transplant mechanism (see [`../param/076_from.md`](../param/076_from.md) § Behavior) — `topic` introduces no new session-management code for this. The first invocation of a given subdirectory name has no session file there yet, so `--from`'s (default: cwd) most recent session is physically copied in (clone). Every subsequent invocation of that same name finds the copy already in place and continues it (`-c`) instead of re-copying.

**Auto-naming is always fresh:** the slug+counter algorithm only ever selects a name that does not yet exist as a subdirectory — an auto-named invocation therefore always clones, never continues. To continue an auto-named topic later, pass its generated name back explicitly via `--subdir`.

`--output-format stream-json` streaming behavior is identical to `run` — see [`run`'s Notes](01_run.md#notes) for details.

### Related Commands

| # | Command | Relationship |
|---|---------|---------------|
| 1 | [`run`](01_run.md) | `topic` delegates to `run`'s execution path with `--subdir`'s default changed |
| 2 | [`ask`](05_ask.md) | Sibling alias in the same command_group; `topic` differs from `ask` only in `--subdir`'s default |

### Referenced Parameter Groups

| # | Group | Membership | Excluded Params |
|---|-------|------------|-----------------|
| 1 | [Claude-Native Flags](../param_group/01_claude_native_flags.md) | Full | — |
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full (`--subdir` default overridden) | — |
| 3 | [System Prompt](../param_group/03_system_prompt.md) | Full | — |

### Referenced Command Group

| # | Group | Role |
|---|-------|------|
| 1 | [run / ask / topic](../command_group/01_run_ask.md) | Member — delegates to `run`'s handler with `--subdir`'s default overridden |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 22 | [022_session_isolation_subdir.md](../user_story/022_session_isolation_subdir.md) | Developer |
| 28 | [028_session_transplant.md](../user_story/028_session_transplant.md) | Developer |
| 30 | [030_topic_creation.md](../user_story/030_topic_creation.md) | Developer |

---

**Category:** Task execution
**Complexity:** 30
**API Requirement:** Write
**Idempotent:** No
**Risk Level:** Low
