# CLI Parameter: --output-file

Write captured stdout to a file in addition to printing it to stdout (tee
behavior). The runner captures the full subprocess output first, then writes it
to the file and prints it to stdout in one operation.

- **Type:** string (file path)
- **Default:** — (output goes only to stdout)
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)
- **JSON Key:** `"output-file"`

```sh
clr ask "Summarize this module" --output-file /tmp/summary.txt
clr --output-file patch.txt "Fix the authentication bug"
clr --file schema.json --output-file types.rs "Generate Rust types"
```

**Note:** Write errors (permission denied, directory does not exist) exit with
code 1 and emit the OS error to stderr. The subprocess is not affected.

**Note:** When combined with `--strip-fences`, the fence-stripped text is both
written to the file and printed to stdout — the same content reaches both
destinations.

**Note:** `--output-file` is orthogonal to `--file` — `--file` feeds input to
the subprocess stdin; `--output-file` captures subprocess stdout to a file.

**Note:** In dry-run mode the file is NOT created; the path is shown in the
assembled command description.

**Env var:** `CLR_OUTPUT_FILE` — accepts a file path string; applied when
`--output-file` is absent from the CLI.

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | 21 other params |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | — | — |
| 5 | [`ask`](../command/05_ask.md) | — | — |

### See Also

- [`--strip-fences`](026_strip_fences.md) — strips markdown fences before writing
- [`--file`](025_file.md) — stdin input (orthogonal)

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 23 | [023_output_file_capture.md](../user_story/023_output_file_capture.md) | Developer |
| 2 | [002_print_mode_capture.md](../user_story/002_print_mode_capture.md) | Developer |
