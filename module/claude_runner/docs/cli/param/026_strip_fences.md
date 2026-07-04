# CLI Parameter: --strip-fences

Post-processes captured stdout: removes the first and last markdown code
fence lines (lines matching `` ``` `` optionally followed by a language tag,
e.g. `` ```rust ``). All content between the opening and closing fence lines
is emitted unchanged. If no fence pair is found, stdout passes through
unmodified.

- **Type:** bool
- **Default:** false (fence stripping disabled)
- **Command:** [`run`](../command/01_run.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)
- **JSON Key:** `"strip-fences"`

```sh
clr --strip-fences "Generate a Rust struct for Config"
clr --file schema.json --strip-fences "Convert this JSON to a Rust type"
clr -p --strip-fences "Write a shell script to back up /etc"
```

**Note:** Only the outermost fence pair is stripped. If the output contains
multiple code blocks, the first `` ``` `` line and the last `` ``` `` line are
used as boundaries; everything between them is emitted including any
interior fence pairs.

**Note:** If the output contains no fence pair (neither opening nor closing
`` ``` `` line), the flag has no effect and the full stdout is emitted as-is.

**Note:** The fence-stripping step runs after the subprocess completes and
its captured stdout is available in memory. It has no effect in dry-run mode
(no subprocess runs in dry-run mode).

**Env var:** `CLR_STRIP_FENCES` — accepts `1` or `true` (case-insensitive); applied when
`--strip-fences` is absent from the CLI. `CLR_STRIP_FENCES=1 clr -p "task"` is equivalent
to `clr -p --strip-fences "task"`.

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| bool | Primitive | bool | present/absent |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | 16 other params |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | false | — |
| 5 | [`ask`](../command/05_ask.md) | false | — |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 2 | [002_print_mode_capture.md](../user_story/002_print_mode_capture.md) | Developer |
| 11 | [011_file_input.md](../user_story/011_file_input.md) | Developer |
| 12 | [012_code_block_extraction.md](../user_story/012_code_block_extraction.md) | Developer |
| 13 | [013_structured_json_pipeline.md](../user_story/013_structured_json_pipeline.md) | Developer |
