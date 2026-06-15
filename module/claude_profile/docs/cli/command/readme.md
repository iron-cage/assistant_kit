# Commands

`clp` CLI commands organized by namespace.

| File | Responsibility |
|------|----------------|
| [003_meta.md](003_meta.md) | Meta-commands: `.`, `.help`, `--version` flag |
| [001_account.md](001_account.md) | Account namespace: `.accounts`, `.account.save`, `.account.use`, `.account.delete`, `.account.limits`, `.account.relogin`, `.account.rotate`, `.account.renewal`, `.account.inspect`, `.account.assign`, `.account.unclaim` |
| [005_token.md](005_token.md) | Token namespace: `.token.status` |
| [002_credentials.md](002_credentials.md) | Credentials namespace: `.credentials.status` |
| [006_usage.md](006_usage.md) | Usage namespace: `.usage` |
| [004_paths.md](004_paths.md) | Paths namespace: `.paths` |
| [007_model.md](007_model.md) | Model namespace: `.model` |

**Total:** 18 commands (17 visible + 1 hidden)

### All Commands

| # | Command | Purpose | Params | Example |
|---|---------|---------|--------|---------|
| 1 | `.` | Show help information (hidden dot-shorthand) | 0 | `clp .` |
| 2 | `.help` | Display command reference and usage examples | 0 | `clp .help` |
| 3 | `.accounts` | List all saved accounts or show a single named account | 18 | `clp .accounts` |
| 4 | `.account.save` | Save current credentials as a named account profile | 5 | `clp .account.save name::alice@acme.com` |
| 5 | `.account.use` | Switch active account by name with atomic credential rotation | 7 | `clp .account.use name::alice@home.com` |
| 6 | `.account.delete` | Delete a saved account from the account store | 3 | `clp .account.delete name::alice@oldco.com` |
| 7 | `.token.status` | Show active OAuth token expiry classification | 3 | `clp .token.status` |
| 8 | `.paths` | Show all resolved ~/.claude/ canonical file paths | 3 | `clp .paths` |
| 9 | `.usage` | Show live rate-limit quota for all saved accounts | 25 | `clp .usage` |
| 10 | `.credentials.status` | Show live credential metadata without account store dependency | 18 | `clp .credentials.status` |
| 11 | `.account.limits` | Show rate-limit utilization for the active or named account | 3 | `clp .account.limits name::alice@acme.com` |
| 12 | `.account.relogin` | Force browser re-authentication for a named account | 3 | `clp .account.relogin name::carol@example.com` |
| 13 | `.account.rotate` | Auto-rotate to the best inactive account by token expiry | 2 | `clp .account.rotate` |
| 14 | `.account.renewal` | Set/clear billing renewal timestamp override for one or all accounts | 6 | `clp .account.renewal name::alice@acme.com from_now::+0m` |
| 15 | `.account.inspect` | Live diagnostic inspection of identity, subscription, and org fields | 4 | `clp .account.inspect` |
| 16 | `.account.assign` | Write the per-machine active-account marker for any `USER@MACHINE` without credential rotation | 3 | `clp .account.assign name::alice@corp.com` |
| 17 | `.account.unclaim` | Release ownership of a saved account profile (pure metadata; no credential touch) | 3 | `clp .account.unclaim name::alice@acme.com` |
| 18 | `.model` | Get or set the Claude Code session model in `~/.claude/settings.json` | 2 | `clp .model set::opus` |

### Quick Reference

**Required Parameters:**
- `name::` — required on `.account.use`, `.account.delete`, `.account.relogin`, `.account.renewal`, `.account.unclaim`; optional on `.account.save` (inferred), `.accounts`, `.account.limits`, `.account.inspect` (defaults to active account).

**Most-Used Parameters:**
- `format::` — 7 commands (`.accounts`, `.token.status`, `.paths`, `.usage`, `.credentials.status`, `.account.limits`, `.account.inspect`)

**Commands by Parameter Count:**

| Count | Commands |
|-------|----------|
| 0 | `.`, `.help` |
| 2 | `.account.rotate`, `.model` |
| 3 | `.paths`, `.account.delete`, `.token.status`, `.account.limits`, `.account.relogin`, `.account.assign`, `.account.unclaim` |
| 4 | `.account.inspect` |
| 5 | `.account.save` |
| 6 | `.account.renewal` |
| 7 | `.account.use` |
| 18 | `.accounts` |
| 18 | `.credentials.status` |
| 25 | `.usage` |

### See Also

- [../param/](../param/readme.md) — parameter specifications
- [../type/](../type/readme.md) — types used by commands
- [../param_group/](../param_group/readme.md) — parameter group definitions
- [../user_story/](../user_story/readme.md) — user stories referencing these commands
- [../command_noun/](../command_noun/readme.md) — domain noun documentation
- [../command_verb/](../command_verb/readme.md) — domain verb documentation
