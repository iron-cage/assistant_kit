# Feature Doc Entity

### Scope

- **Purpose**: Defines the functional capabilities of `claude_profile` — account credential management and the `clp` CLI.
- **Responsibility**: Documents all functional requirements with their design, acceptance criteria, and test references.
- **In Scope**: FR-6 through FR-22 plus unassigned FR for 017–018; CLI commands and library API surface.
- **Out of Scope**: Quality constraints (→ invariant/), CLI design (→ cli/).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Account Store Initialization](001_account_store_init.md) | Initialize credential store with `$PRO`/`$HOME` path resolution | ✅ |
| 002 | [Save Account](002_account_save.md) | Save current credentials as a named account profile | ✅ |
| 003 | [List Accounts](003_account_list.md) | List all stored accounts with token state and metadata | ✅ |
| 004 | [Switch Account](004_account_use.md) | Atomically switch the active credential set to a named account | ✅ |
| 005 | [Delete Account](005_account_delete.md) | Remove a named account with an active-account safety guard | ✅ |
| 006 | [Token Status](006_token_status.md) | Classify the active OAuth token as Valid, ExpiringSoon, or Expired | ✅ |
| 007 | [File Topology](007_file_topology.md) | Expose all `~/.claude/` canonical paths via a typed API | ✅ |
| 008 | [Auto Rotate](008_auto_rotate.md) | Rotate to the inactive account with the highest token expiry | ✅ |
| 009 | [Token Usage Reporting](009_token_usage.md) | Show live quota utilization for all saved accounts via API | ✅ |
| 010 | [Persistent Storage Path](010_persistent_storage.md) | Resolve persistent storage root and credential store from `$PRO` / `$HOME` | ✅ |
| 011 | [Account Status by Name](011_account_status_by_name.md) | Inspect any stored account's token state without switching | ✅ |
| 012 | [Live Credentials Status](012_live_credentials_status.md) | Show credential metadata with no account store dependency | ✅ |
| 013 | [Account Rate-Limit Utilization](013_account_limits.md) | Show rate-limit utilization via live HTTP response headers | ✅ |
| 014 | [Rich Account Metadata](014_rich_account_metadata.md) | Expose `oauthAccount` and model fields in `.credentials.status` and `.accounts` | ✅ |
| 015 | [Account Name Shortcut Syntax](015_name_shortcut_syntax.md) | Positional bare arg and prefix resolution for `name::` on four account commands | 🚧 |
| 016 | [Current Account Awareness](016_current_account_awareness.md) | Current account detection via token match; divergence display in `.accounts` and `.usage` | 🚧 |
| 017 | [Expired Token Refresh via Isolated Subprocess](017_token_refresh.md) | `refresh::` parameter; retry-on-auth-error via `run_isolated()`; credential write-back | ✅ |
| 018 | [Live Quota Monitor Mode](018_live_monitor.md) | `live::`, `interval::`, `jitter::` parameters; continuous refresh with staggered fetches and countdown footer | 🎯 |
| — | [procedure.md](procedure.md) | Workflow for creating and updating feature doc instances | ✅ |
