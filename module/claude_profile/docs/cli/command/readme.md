# Commands

`clp` CLI commands organized by namespace.

| File | Responsibility |
|------|----------------|
| [003_meta.md](003_meta.md) | Meta-commands: `.`, `.help`, `--version` flag |
| [001_account.md](001_account.md) | Account namespace: `.accounts`, `.account.save`, `.account.use`, `.account.delete`, `.account.limits`, `.account.relogin`, `.account.renewal`, `.account.inspect` |
| [005_token.md](005_token.md) | Token namespace: `.token.status` |
| [002_credentials.md](002_credentials.md) | Credentials namespace: `.credentials.status` |
| [006_usage.md](006_usage.md) | Usage namespace: `.usage` |
| [004_paths.md](004_paths.md) | Paths namespace: `.paths` |
| [007_model.md](007_model.md) | Model namespace: `.model`, `.model.select` |
| [008_models.md](008_models.md) | Models discovery: `.models` |

**Total:** 20 commands (15 visible + 2 hidden + 1 DEPRECATED: `.account.rotate` (Feature 038) + 2 REMOVED: `.account.assign`, `.account.unclaim`)

### All Commands

| # | Command | Purpose | Params | Example |
|---|---------|---------|--------|---------|
| 1 | `.` | Show help information (hidden dot-shorthand) | 0 | `clp .` |
| 2 | `.help` | Display command reference and usage examples | 0 | `clp .help` |
| 3 | `.accounts` | List all saved accounts or show a single named account | 32 | `clp .accounts` |
| 4 | `.account.save` | Save current credentials as a named account profile | 5 | `clp .account.save name::alice@acme.com` |
| 5 | `.account.use` | Switch active account by name with atomic credential rotation | 8 | `clp .account.use name::alice@home.com` |
| 6 | `.account.delete` | Delete a saved account from the account store | 3 | `clp .account.delete name::alice@oldco.com` |
| 7 | `.token.status` | Show active OAuth token expiry classification | 3 | `clp .token.status` |
| 8 | `.paths` | Show all resolved ~/.claude/ canonical file paths | 3 | `clp .paths` |
| 9 | `.usage` | Show live rate-limit quota for all saved accounts | 33 | `clp .usage` |
| 10 | `.credentials.status` | Show live credential metadata without account store dependency | 18 | `clp .credentials.status` |
| 11 | `.account.limits` | Show rate-limit utilization for the active or named account | 3 | `clp .account.limits name::alice@acme.com` |
| 12 | `.account.relogin` | Force browser re-authentication for a named account | 3 | `clp .account.relogin name::carol@example.com` |
| 13 | `.account.rotate` | **DEPRECATED** — hidden redirector; exits 1 with notice to use `.usage rotate::1` | 0 | `clp .account.rotate` |
| 14 | `.account.renewal` | Set/clear billing renewal timestamp override for one or all accounts | 6 | `clp .account.renewal name::alice@acme.com from_now::+0m` |
| 15 | `.account.inspect` | Live diagnostic inspection of identity, subscription, and org fields | 4 | `clp .account.inspect` |
| 16 | `.account.assign` | **REMOVED** (Feature 037) — writes per-machine active marker only; use `.accounts assignee::USER@MACHINE name::X` | 0 | `clp .account.assign name::alice@acme.com` |
| 17 | `.account.unclaim` | **REMOVED** (Feature 037; absorbed param further REMOVED Feature 064) — releases account ownership; use `.accounts owner::0 name::X` | 0 | `clp .account.unclaim name::alice@acme.com` |
| 18 | `.model` | Get or set the Claude Code session model in `~/.claude/settings.json` | 2 | `clp .model set::opus` |
| 19 | `.models` | List available Claude models via live API or static offline catalog | 3 | `clp .models offline::1` |
| 20 | `.model.select` | Get/set/reset subprocess model preference in `~/.clr/prefs.json` | 3 | `clp .model.select id::claude-opus-4-8` |

### Quick Reference

**Required Parameters:**
- `name::` — required on `.account.use`, `.account.delete`, `.account.relogin`, `.account.renewal`; optional on `.account.save` (inferred), `.accounts`, `.account.limits`, `.account.inspect` (defaults to active account). For ownership release, use `.accounts owner::0 name::X` (Feature 064).

**Most-Used Parameters:**
- `format::` — 9 commands (`.accounts`, `.token.status`, `.paths`, `.usage`, `.credentials.status`, `.account.limits`, `.account.inspect`, `.models`, `.model.select`)

**Commands by Parameter Count:**

| Count | Commands |
|-------|----------|
| 0 | `.`, `.help` |
| 2 | `.model` |
| 3 | `.paths`, `.account.delete`, `.token.status`, `.account.limits`, `.account.relogin`, `.models`, `.model.select` |
| 4 | `.account.inspect` |
| 5 | `.account.save` |
| 6 | `.account.renewal` |
| 8 | `.account.use` |
| 18 | `.credentials.status` |
| 32 | `.accounts` |
| 33 | `.usage` |

### See Also

- [../param/](../param/readme.md) — parameter specifications
- [../type/](../type/readme.md) — types used by commands
- [../param_group/](../param_group/readme.md) — parameter group definitions
- [../user_story/](../user_story/readme.md) — user stories referencing these commands
- [../command_noun/](../command_noun/readme.md) — domain noun documentation
- [../command_verb/](../command_verb/readme.md) — domain verb documentation
