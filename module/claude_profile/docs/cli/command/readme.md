# Commands

`clp` CLI commands organized by namespace.

| File | Responsibility |
|------|----------------|
| [003_meta.md](003_meta.md) | Meta-commands: `.`, `.help`, `--version` flag |
| [001_account.md](001_account.md) | Account namespace: `.accounts`, `.account.save`, `.account.use`, `.account.delete`, `.account.limits`, `.account.relogin`, `.account.rotate` |
| [005_token.md](005_token.md) | Token namespace: `.token.status` |
| [002_credentials.md](002_credentials.md) | Credentials namespace: `.credentials.status` |
| [006_usage.md](006_usage.md) | Usage namespace: `.usage` |
| [004_paths.md](004_paths.md) | Paths namespace: `.paths` |

**Total:** 13 commands (11 visible + 2 hidden)

### All Commands

| # | Command | Purpose | Params | Example |
|---|---------|---------|--------|---------|
| 1 | `.` | Show help information (hidden dot-shorthand) | 0 | `clp .` |
| 2 | `.help` | Display command reference and usage examples | 0 | `clp .help` |
| 3 | `.accounts` | List all saved accounts or show a single named account | 12 | `clp .accounts` |
| 4 | `.account.save` | Save current credentials as a named account profile | 2 | `clp .account.save name::alice@acme.com` |
| 5 | `.account.use` | Switch active account by name with atomic credential rotation | 2 | `clp .account.use name::alice@home.com` |
| 6 | `.account.delete` | Delete a saved account from the account store | 2 | `clp .account.delete name::alice@oldco.com` |
| 7 | `.token.status` | Show active OAuth token expiry classification | 2 | `clp .token.status` |
| 8 | `.paths` | Show all resolved ~/.claude/ canonical file paths | 2 | `clp .paths` |
| 9 | `.usage` | Show live rate-limit quota for all saved accounts | 6 | `clp .usage` |
| 10 | `.credentials.status` | Show live credential metadata without account store dependency | 13 | `clp .credentials.status` |
| 11 | `.account.limits` | Show rate-limit utilization for the active or named account | 2 | `clp .account.limits name::alice@acme.com` |
| 12 | `.account.relogin` | Force browser re-authentication for a named account | 2 | `clp .account.relogin name::i3@wbox.pro` |
| 13 | `.account.rotate` | Auto-rotate to the best inactive account by token expiry | 1 | `clp .account.rotate` |

### Quick Reference

**Required Parameters:**
- `name::` — required on `.account.use`, `.account.delete`, `.account.relogin`; optional on `.account.save` (inferred), `.accounts`, `.account.limits`.

**Most-Used Parameters:**
- `format::` — 6 commands

**Commands by Parameter Count:**

| Count | Commands |
|-------|----------|
| 0 | `.`, `.help` |
| 1 | `.account.rotate` |
| 2 | `.paths`, `.account.save`, `.account.use`, `.account.delete`, `.token.status`, `.account.limits`, `.account.relogin` |
| 6 | `.usage` |
| 12 | `.accounts` |
| 13 | `.credentials.status` |

### See Also

- [../param/](../param/readme.md) — parameter specifications
- [../type/](../type/readme.md) — types used by commands
- [../param_group/](../param_group/readme.md) — parameter group definitions
