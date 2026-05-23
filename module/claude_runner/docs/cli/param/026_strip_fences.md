# Parameter :: 26. `--strip-fences`

Post-processes captured stdout: removes the first and last markdown code
fence lines (lines matching `` ``` `` optionally followed by a language tag,
e.g. `` ```rust ``). All content between the opening and closing fence lines
is emitted unchanged. If no fence pair is found, stdout passes through
unmodified.

- **Type:** bool
- **Default:** false (fence stripping disabled)
- **Command:** [`run`](../001_command.md#command--1-run)
- **Group:** [Runner Control](../004_param_group.md#group--2-runner-control)

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
