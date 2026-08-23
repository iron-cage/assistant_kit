# Environment Parameters

### Scope

- **Purpose**: Environment variable reference for the `clj` CLI.
- **Responsibility**: All environment variables that affect `clj` behavior — names, CLI equivalents, defaults, precedence, and consuming commands.
- **In Scope**: All env vars read by `clj` at startup or during command execution.
- **Out of Scope**: CLI parameter reference (→ `param/`), command reference (→ `command/`).

### Precedence

CLI parameters always override environment variables. Resolution order:

1. CLI `param::value` argument (highest priority)
2. Environment variable
3. Built-in default

### All Environment Parameters (3 total)

| # | Env Variable | CLI Equivalent | Default | Commands |
|---|-------------|----------------|---------|----------|
| 1 | `CLR_JOURNAL_DIR` | `journal_dir::` | `~/.clr/journal/` | .list, .tail, .search, .stats, .status |
| 2 | `NO_COLOR` | `no_color::1` | -- (colors enabled) | .list, .tail, .stats |
| 3 | `CLJ_PORT` | `port::` | `0` (OS-assigned) | .serve |

### Details

#### CLR_JOURNAL_DIR

Shared with `clr` — both the writer (`clr`) and viewer (`clj`) use the
same environment variable to locate the journal directory. This is the
single canonical env var for journal location.

```bash
export CLR_JOURNAL_DIR=/var/log/clr/journal
clj .list                            # Reads from /var/log/clr/journal
clj .list journal_dir::~/.clr/journal/  # CLI override
```

**Consumed by:** `.list`, `.tail`, `.search`, `.stats`, `.status`

#### NO_COLOR

Follows the [no-color.org](https://no-color.org) convention. When set
to any non-empty value, ANSI color codes are suppressed in table output.
Equivalent to `no_color::1` on every command.

```bash
NO_COLOR=1 clj .list                  # Plain text output
NO_COLOR=1 clj .stats since::7d      # No colors in stats table
```

**Consumed by:** `.list`, `.tail`, `.stats`

#### CLJ_PORT

Default port for the embedded web viewer. Overridden by `port::` CLI param.
When neither is set the port resolves to `0` and the OS assigns an ephemeral
one — there is no fixed fallback port; read the actual value from the
`Listening on http://localhost:{port}` startup line.

```bash
export CLJ_PORT=9090
clj .serve                            # Binds to port 9090
clj .serve port::8080                 # CLI override to 8080
unset CLJ_PORT; clj .serve            # OS-assigned port
```

**Consumed by:** `.serve`
