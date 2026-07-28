# CLI Parameter: --mode

Filter `clr ps` output by session execution mode. Sessions launched with the
control-protocol trio `--input-format stream-json --output-format stream-json --verbose`
are classified as `query`; else sessions launched with `--print` (or `-p`) are classified
as `print`; all others are classified as `interactive`.

- **Type:** enum (`all` | `interactive` | `print` | `query`)
- **Default:** `all`
- **Command:** [`ps`](../command/06_ps.md)
- **JSON Key:** — (ps subcommand; not supported by `--args-file`)

```sh
clr ps --mode interactive    # show only interactive (TTY) sessions
clr ps --mode print          # show only print-mode (headless) sessions
clr ps --mode query          # show only clr-query control sessions
clr ps --mode all            # show all three (default behavior)
clr ps -m print              # short form
CLR_PS_MODE=interactive clr ps  # env-var equivalent
```

**Note:** Mode detection reads the NUL-delimited `/proc/{pid}/cmdline` arguments
for each Claude process, in priority order: if the arguments contain the adjacent
pairs `--input-format stream-json` and `--output-format stream-json` plus a bare
`--verbose` flag, the session is classified as `query`; else if the argument list
contains `--print` or `-p`, the session is classified as `print`; otherwise
`interactive`. The detection examines individual arguments (NUL-separated fields),
not the joined cmdline string.

**Note:** Invalid values (anything other than `all`, `interactive`, `print`, `query`) cause
`clr ps` to exit 1 with an error message to stderr.

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 5 | [Session Listing](../param_group/05_session_listing.md) | Full | `--columns`, `--wide` |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 6 | [`ps`](../command/06_ps.md) | `all` | Filters the active sessions table; queued table unaffected |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 26 | [026_session_listing.md](../user_story/026_session_listing.md) | Developer / CI operator |
