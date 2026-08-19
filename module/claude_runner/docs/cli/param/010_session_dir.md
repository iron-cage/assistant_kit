# CLI Parameter: --session-dir

**DEPRECATED — accepted but fully inert.** claude >= 2.x ignores the
`CLAUDE_CODE_SESSION_DIR` environment variable this parameter used to export
(proven by BUG-490's control experiment; [Contract B23](../../../../../contract/claude_code/docs/behavior/023_b23_session_dir_override.md)
was NEG-ONLY from introduction), so a raw storage-path override cannot work.
Fix(BUG-493) removed the export and the parameter's role in `-c` gating and
transplant planning. Sessions always use the working directory's own project
storage; use [`--from <dir>`](076_from.md) to continue another project's session.

- **Type:** [`DirectoryPath`](../type/02_directory_path.md)
- **Default:** — (no effect either way)
- **Command:** [`run`](../command/01_run.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)
- **Validation:** requires a value
- **JSON Key:** `"session-dir"`

### Deprecation Behavior

- Still parsed from all three sources — CLI flag, `CLR_SESSION_DIR` env var,
  JSON `"session-dir"` key — so existing invocations don't hard-fail.
- Applies **no effect**: no env export, no influence on `-c` injection, no
  influence on the `--from` transplant plan.
- Emits exactly one stderr warning when given (unconditional — not gated by
  `--quiet`): `[Runner] warning: --session-dir is deprecated and has no effect: ...`

```sh
# Accepted, warns on stderr, changes nothing:
clr "Fix bug" --session-dir /tmp/my-sessions
# The working replacement for cross-project continuation:
clr --from /path/to/other/project "Fix bug"
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

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [001_interactive_repl.md](../user_story/001_interactive_repl.md) | Developer |
| 5 | [005_project_specific_execution.md](../user_story/005_project_specific_execution.md) | Developer |
