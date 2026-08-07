# Commands

`clp` CLI commands organized by namespace.

| File | Responsibility |
|------|----------------|
| [003_meta.md](003_meta.md) | Meta-commands: `.`, `.help`, `--version` flag |
| [001_account.md](001_account.md) | Account namespace: `.accounts`, `.account.save`, `.account.use`, `.account.delete`, `.account.limits`, `.account.relogin`, `.account.renewal`, `.account.inspect` |
| [005_token.md](005_token.md) | **DEPRECATED** — Token namespace: `.token.status` (removed; see `.credentials.status`) |
| [002_credentials.md](002_credentials.md) | Credentials namespace: `.credentials.status` |
| [006_usage.md](006_usage.md) | Usage namespace: `.usage` |
| [004_paths.md](004_paths.md) | Paths namespace: `.paths` |
| [007_model.md](007_model.md) | Model namespace: `.model` (unified session + subprocess, `scope::`-routed); `.model.select` (removed, merged into `.model`) |
| [008_models.md](008_models.md) | Models discovery: `.models` |
| [009_provider.md](009_provider.md) | Provider namespace: `.provider.select` |

**Total:** 21 commands (14 visible + 2 hidden + 1 DEPRECATED: `.account.rotate` (Feature 038) + 4 REMOVED: `.account.assign`, `.account.unclaim`, `.token.status`, `.model.select` (Feature 035))

### All Commands

| # | Command | Purpose | Params | Example |
|---|---------|---------|--------|---------|
| 1 | `.` | Show help information (hidden dot-shorthand) | 0 | `clp .` |
| 2 | `.help` | Display command reference and usage examples | 0 | `clp .help` |
| 3 | `.accounts` | List all saved accounts or show a single named account | 32 | `clp .accounts` |
| 4 | `.account.save` | Save current credentials as a named account profile | 5 | `clp .account.save name::alice@acme.com` |
| 5 | `.account.use` | Switch active account by name with atomic credential rotation | 8 | `clp .account.use name::alice@home.com` |
| 6 | `.account.delete` | Delete a saved account from the account store | 3 | `clp .account.delete name::alice@oldco.com` |
| 7 | `.token.status` | **REMOVED** — use `.credentials.status`'s `token`/`expires` fields with `threshold::` | 0 | `clp .token.status` |
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
| 18 | `.model` | Get/set/reset model and effort for the session (`~/.claude/settings.json`) or subprocess (`~/.clr/config.toml`) store, via `scope::` | 6 | `clp .model model::opus` |
| 19 | `.models` | List available Claude models via live API or static offline catalog | 3 | `clp .models offline::1` |
| 20 | `.model.select` | **REMOVED** (Feature 035) — merged into `.model`; use `.model scope::subprocess model::VALUE` | 0 | `clp .model.select id::claude-opus-4-8` |
| 21 | `.provider.select` | Get/set/reset global inference provider in `~/.clr/config.toml` | 3 | `clp .provider.select id::kimi` |

### Quick Reference

**Required Parameters:**
- `name::` — required on `.account.use`, `.account.delete`, `.account.relogin`, `.account.renewal`; optional on `.account.save` (inferred), `.accounts`, `.account.limits`, `.account.inspect` (defaults to active account). For ownership release, use `.accounts owner::0 name::X` (Feature 064).

**Most-Used Parameters:**
- `format::` — 9 commands (`.accounts`, `.paths`, `.usage`, `.credentials.status`, `.account.limits`, `.account.inspect`, `.models`, `.model`, `.provider.select`)

**Commands by Parameter Count:**

| Count | Commands |
|-------|----------|
| 0 | `.`, `.help` |
| 3 | `.paths`, `.account.delete`, `.account.limits`, `.account.relogin`, `.models`, `.provider.select` |
| 4 | `.account.inspect` |
| 5 | `.account.save` |
| 6 | `.account.renewal`, `.model` |
| 8 | `.account.use` |
| 19 | `.credentials.status` |
| 32 | `.accounts` |
| 33 | `.usage` |

### See Also

- [../param/](../param/readme.md) — parameter specifications
- [../type/](../type/readme.md) — types used by commands
- [../param_group/](../param_group/readme.md) — parameter group definitions
- [../user_story/](../user_story/readme.md) — user stories referencing these commands
- [../command_noun/](../command_noun/readme.md) — domain noun documentation
- [../command_verb/](../command_verb/readme.md) — domain verb documentation
