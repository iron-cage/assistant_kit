# CLI Parameter: --args-file

Load clr parameters from a JSON file, overriding CLR_* env vars but below explicit CLI flags.
Enables repeatable automation configurations without long flag lists.

- **Type:** [`FilePath`](../type/12_file_path.md)
- **Default:** — (none; no JSON config loaded)
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md), [`isolated`](../command/03_isolated.md), [`refresh`](../command/04_refresh.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)
- **Env:** `CLR_ARGS_FILE` (path; applied only when `--args-file` absent from CLI)
- **JSON Key:** `"args-file"` *(not re-processed inside a JSON source; chaining not supported)*

```sh
clr --args-file fast.json "Fix the bug"          # load params from fast.json
CLR_ARGS_FILE=~/.clr/defaults.json clr "task"   # env var equivalent
cat fast.json | clr -p "task"                    # stdin JSON pipe (auto-detected)
clr --args-file fast.json --dry-run "task"       # inspect merged params
```

**JSON structure:** A flat JSON object where each key is a clr parameter name (without `--`) and the value is the parameter value. All 74 active clr parameters are supported.

```json
{
  "model": "claude-haiku-4-5-20251001",
  "max-sessions": 5,
  "system-prompt": "You are a helpful assistant.",
  "dry-run": false
}
```

**Precedence:** CLI flags > JSON file/stdin > CLR_* env vars > built-in defaults.

**Boolean flags:** JSON `true` activates the flag; `false` is a no-op. Only JSON boolean literals are accepted — string booleans (`"true"`, `"1"`) are rejected.

**Unknown keys:** Unrecognized keys are silently ignored (forward-compatible).

**Error handling:** Non-existent path exits 1 with file-not-found error. Invalid JSON exits 1 with parse error. Both errors emit to stderr before any subprocess is spawned.

**Stdin pipe:** When stdin is not a TTY and begins with `{`, clr auto-detects stdin as a JSON parameter source (equivalent to `--args-file`). `--file` takes priority over stdin JSON detection when both are present.

### Since

Introduced in the JSON Config Loading feature (feature/004_json_config.md).

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | 46 other params |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | — | Applies JSON config before flag parsing |
| 2 | [`isolated`](../command/03_isolated.md) | — | Applies JSON config before flag parsing |
| 3 | [`refresh`](../command/04_refresh.md) | — | Applies JSON config before flag parsing |
| 5 | [`ask`](../command/05_ask.md) | — | Applies JSON config before flag parsing |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 5 | [005_project_specific_execution.md](../user_story/005_project_specific_execution.md) | Developer |
