# CLI Parameter: --session-dir

**Deprecated and inert (BUG-493):** claude ≥2.x ignores the `CLAUDE_CODE_SESSION_DIR`
export this flag used to set, for both reads and writes. Setting `--session-dir` or
`CLR_SESSION_DIR` to a non-empty value has no effect on where sessions load from or
save to, and emits a deprecation warning (unless `--quiet`) naming the value. It never
suppresses [`--from`](076_from.md)'s transplant — the only mechanism that still works
for cross-loading another project's session history.

- **Type:** [`DirectoryPath`](../type/02_directory_path.md)
- **Default:** — (unset; has no effect even when given a value)
- **Command:** [`run`](../command/01_run.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)
- **Validation:** requires a value
- **Env var:** `CLR_SESSION_DIR`
- **JSON Key:** `"session-dir"`

```sh
# No longer has any effect on session storage — retained only to emit the
# deprecation warning below. Use --from to cross-load another project's session.
clr "Fix bug" --session-dir /tmp/my-sessions
```

```
Warning: --session-dir/CLR_SESSION_DIR (/tmp/my-sessions) is deprecated and has no effect;
claude ignores this override and continues using its own session storage.
Use --from to seed continuation from another project's session history instead.
```

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`DirectoryPath`](../type/02_directory_path.md) | Semantic | String | valid filesystem path |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | 16 other params |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | — | — |
| 5 | [`ask`](../command/05_ask.md) | — | — |
| 11 | [`topic`](../command/11_topic.md) | — | Identical to `ask`; delegates to `run`'s handler |

### Referenced Parameters

| # | Parameter | Relationship |
|---|-----------|--------------|
| 076 | [`--from`](076_from.md) | The working replacement for cross-loading; never suppressed by this deprecated flag |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [001_interactive_repl.md](../user_story/001_interactive_repl.md) | Developer |
| 5 | [005_project_specific_execution.md](../user_story/005_project_specific_execution.md) | Developer |
