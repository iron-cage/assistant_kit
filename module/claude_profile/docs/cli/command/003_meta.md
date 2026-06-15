# Commands :: Meta

Meta-commands and flags that control CLI-level behaviour rather than account operations.

---

### Meta-flag :: `--version` / `-V`

Print the binary name and version string, then exit. This flag takes priority over all commands and parameters — it wins regardless of argv order. Not a command (does not appear in `.help` listing).

- **Aliases:** `-V`
- **Exit:** 0 (always)
- **Output:** `clp X.Y.Z` (one line on stdout; stderr is empty)
- **Implementation:** `src/lib.rs::cli::run()`

**Algorithm (1 step):** Read compiled version constant; print `clp X.Y.Z` to stdout; exit 0.

**Examples:**

```bash
clp --version   # → "clp 0.12.3"
clp -V          # → identical output
```

---

### Command :: 1. `.`

Hidden alias that triggers `print_usage()` at the adapter layer — identical to typing `.help`.
The `.` command is registered in the registry with `hidden_from_list(true)` so it never
appears in its own output; its registered handler (`dot_routine`) is never invoked because
the adapter intercepts `.` before the unilang pipeline.

-- **Parameters:** none accepted (trailing `key::value` tokens are silently ignored)
-- **Exit:** 0 (always)

**Syntax:**

```bash
clp .
```

**Output (non-TTY, ANSI stripped):**

```
Usage: clp <command>

Manage Claude Code account credentials and token state.

Commands:

  Account management
    .accounts             List all saved accounts
    .account.save         Save current credentials as a named profile
    .account.use          Switch the active account
    .account.delete       Delete a saved account
    .account.limits       Show rate-limit utilization (one account)
    .account.relogin      Re-authenticate via browser login
    .account.rotate       Auto-rotate to the best inactive account
    .account.renewal      Set or clear billing renewal timestamp override
    .account.inspect      Show identity, subscription, and org fields
    .account.assign       Write per-machine active-account marker

  Status & info
    .credentials.status   Show live credential metadata
    .token.status         Show OAuth token expiry classification
    .paths                Show all resolved ~/.claude/ paths
    .usage                Show live quota for all saved accounts
    .model                Get or set session model in settings.json

Options:
  format::text|json     Output format (default: text)
  dry::bool             Dry-run preview (no changes)
  name::EMAIL           Account name

Examples:
  clp .accounts
  clp .account.use alice@acme.com
  clp .usage
  clp .credentials.status
```

**Algorithm (1 step):** Call `print_usage()`; render grouped command list to stdout (ANSI colour on TTY, stripped in pipe); exit 0.

**Notes:**
- On a TTY, group headers and command names are rendered in ANSI colour (yellow/bold-cyan); piped output strips ANSI codes automatically (`std::io::IsTerminal`).
- Commands are grouped into "Account management" and "Status & info"; no per-command parameter listings are shown at this level of abstraction.
- Per-command parameter details are available via `clp <command>.help` (e.g., `clp .accounts.help`).
- Implemented in `src/lib.rs::cli::print_usage()`.

---

### Command :: 2. `.help`

Pre-registered by the unilang `CommandRegistry`. At the adapter layer, `.help` (and bare `help`) set `needs_help=true` which intercepts execution before the unilang pipeline, causing `print_usage()` to run — producing output byte-identical to `clp .`.

-- **Parameters:** none accepted (trailing `key::value` tokens are silently ignored)
-- **Exit:** 0 (always)

**Syntax:**

```bash
clp .help
clp help
```

**Algorithm (1 step):** Call `print_usage()` (identical to `.`); exit 0.

**Notes:**
- Output is identical to `clp .` (both call `print_usage()`).
- `clp .help` is the canonical help invocation; `clp .` is a convenience alias.
