# Parameter :: 25. `--file`

Path to a file whose content is piped as standard input to the `claude`
subprocess. Functionally equivalent to `cat <path> | clr -p "message"` but
without requiring a shell pipeline. The file is opened by the runner at
subprocess spawn time — not by `claude` itself.

- **Type:** [`FilePath`](../type.md#type--12-filepath)
- **Default:** — (unset; subprocess receives no stdin)
- **Command:** [`run`](../command.md#command--1-run)
- **Group:** [Runner Control](../param_group.md#group--2-runner-control)

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
