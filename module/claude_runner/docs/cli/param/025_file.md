# CLI Parameter: --file

Path to a file whose content is piped as standard input to the `claude`
subprocess. Functionally equivalent to `cat <path> | clr -p "message"` but
without requiring a shell pipeline. The file is opened by the runner at
subprocess spawn time — not by `claude` itself.

- **Type:** [`FilePath`](../type/12_file_path.md)
- **Default:** — (unset; subprocess receives no stdin)
- **Command:** [`run`](../command/01_run.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)

```sh
clr --file notes.md "Summarise the above"
clr --file /tmp/diff.txt -p "Review this diff"
clr --file schema.json --model sonnet "Generate types from this schema"
```

**Note:** The file must exist and be readable at invocation time. If the
file cannot be opened, `clr` exits with an error message including the path
and OS error.

**Note:** Paths are resolved relative to the caller's working directory (after
any `--dir` change is applied).

**Note:** `--file` is distinct from `--json-schema` — `--file` feeds raw
file bytes to stdin; `--json-schema` injects a JSON Schema string as a
structured-output constraint forwarded to `claude`.

**Env var:** `CLR_FILE` — accepts a file path string; applied when `--file` is absent from
the CLI. `CLR_FILE=/path/to/file clr "task"` is equivalent to `clr --file /path/to/file "task"`.

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`FilePath`](../type/12_file_path.md) | Semantic | String | file must exist and be readable |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | 16 other params |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | — | — |
| 2 | [`isolated`](../command/02_isolated.md) | — | Pre-spawn existence check (exit 1 if missing); uses `run_isolated_with_stdin_file()` code path (TSK-330) |
| 5 | [`ask`](../command/05_ask.md) | — | — |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 11 | [011_file_input.md](../user_story/011_file_input.md) | Developer |
| 12 | [012_code_block_extraction.md](../user_story/012_code_block_extraction.md) | Developer |
| 13 | [013_structured_json_pipeline.md](../user_story/013_structured_json_pipeline.md) | Developer |
| 15 | [015_ask_mode.md](../user_story/015_ask_mode.md) | Developer |
