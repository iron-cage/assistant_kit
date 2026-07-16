# Feature Doc Entity

### Scope

- **Purpose**: Defines the functional capabilities of `claude_profile` — account credential management and the `clp` CLI.
- **Responsibility**: Documents all functional requirements with their design, acceptance criteria, and test references.
- **In Scope**: feature/001 through feature/040, feature/061 through feature/070 — full functional capability set for claude_profile and the clp CLI.
- **Out of Scope**: Quality constraints (→ invariant/), CLI design (→ cli/).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| — | [procedure](procedure.md) | Workflow for maintaining feature instances | ✅ |
| 001 | [Account Store Initialization](001_account_store_init.md) | Initialize credential store with `$PRO`/`$HOME` path resolution | ✅ |
| 002 | [Save Account](002_account_save.md) | Save current credentials as a named account profile | ✅ |
| 003 | [List Accounts](003_account_list.md) | List all stored accounts with token state and metadata | ✅ |
| 004 | [Switch Account](004_account_use.md) | Atomically switch the active credential set to a named account | ✅ |
| 005 | [Delete Account](005_account_delete.md) | Remove a named account with an active-account safety guard | ✅ |
| 006 | [Token Status](006_token_status.md) | Classify the active OAuth token as Valid, ExpiringSoon, or Expired | ✅ |
| 007 | [File Topology](007_file_topology.md) | Expose all `~/.claude/` canonical paths via a typed API | ✅ |
| 008 | [Auto Rotate](008_auto_rotate.md) | **DEPRECATED** — replaced by `.usage rotate::1` (feature 038); `auto_rotate()` API and `.account.rotate` CLI dropped | ❌ |
| 009 | [Token Usage Reporting](009_token_usage.md) | Show live quota utilization for all saved accounts via API | ✅ |
| 010 | [Persistent Storage Path](010_persistent_storage.md) | Resolve persistent storage root and credential store from `$PRO` / `$HOME` | ✅ |
| 011 | [Account Status by Name](011_account_status_by_name.md) | Inspect any stored account's token state without switching | ✅ |
| 012 | [Live Credentials Status](012_live_credentials_status.md) | Show credential metadata with no account store dependency | ✅ |
| 013 | [Account Rate-Limit Utilization](013_account_limits.md) | Show rate-limit utilization via live HTTP response headers | ✅ |
| 014 | [Rich Account Metadata](014_rich_account_metadata.md) | Expose `oauthAccount` and model fields in `.credentials.status` and `.accounts` | ✅ |
| 015 | [Account Name Shortcut Syntax](015_name_shortcut_syntax.md) | Positional bare arg and prefix resolution for `name::` on five account commands | ✅ |
| 016 | [Current Account Awareness](016_current_account_awareness.md) | Current account detection via token match; divergence display in `.accounts` and `.usage` | ✅ |
| 017 | [Expired Token Refresh via Isolated Subprocess](017_token_refresh.md) | `refresh::` parameter; retry-on-auth-error via `account::refresh_account_token()`; credential write-back | ✅ |
| 018 | [Live Quota Monitor Mode](018_live_monitor.md) | `live::`, `interval::`, `jitter::` parameters; continuous refresh with staggered fetches and countdown footer | ✅ |
| 019 | [Browser Re-Authentication for Named Account](019_account_relogin.md) | `.account.relogin` — spawn `claude` with inherited TTY to refresh a dead `refreshToken`; credential write-back and active restore | ✅ |
| 020 | [Usage Sort Strategies](020_usage_sort_strategies.md) | Row ordering and footer recommendation in `.usage` — `sort::`, `desc::`, `prefer::` with `renew` (default), `name`, `renews` strategies; single-strategy footer | ✅ |
| 021 | [Extended Snapshot Fields](021_extended_snapshot_fields.md) | `tagged_id`, `uuid`, `capabilities` from existing `{name}.json`; `uuid::` and `capabilities::` opt-in params | ✅ |
| 022 | [Org Identity Snapshot](022_org_identity_snapshot.md) | Org identity via endpoint 005 at save-time into `{name}.json`; `org_uuid::` and `org_name::` opt-in params | ✅ |
| 023 | [Next Account Recommendation Strategies](023_next_account_strategies.md) | **DEPRECATED** — absorbed into feature 020; `next::` removed, footer recommendation driven by `sort::` | ❌ |
| 024 | [Session Touch via Isolated Subprocess](024_session_touch.md) | Activate idle accounts' 5h session windows by sending minimal prompt via isolated subprocess; `touch::` parameter | ✅ |
| 025 | [Per-Machine Active Marker](025_per_machine_active_marker.md) | Machine-specific `_active_{hostname}_{user}` marker; exact local-part prefix resolution | ✅ |
| 026 | [Subprocess Model and Effort Control](026_subprocess_model_effort.md) | `imodel::` and `effort::` parameters; auto model selection (haiku default; sonnet when `son_idle=true`); effort resolution | ✅ |
| 027 | [`.account.use` Post-Switch Touch](027_account_use_post_switch_touch.md) | Activate idle 5h window after account switch; `touch::`, `imodel::`, `effort::` on `.account.use` | ✅ |
| 028 | [Usage Row Filtering and Extraction](028_usage_row_filtering.md) | Row-level filters, count/offset pagination, and `get::` single-value extraction for `.usage` output | ✅ |
| 029 | [Account Host and Role Metadata](029_account_host_metadata.md) | Capture host/role labels at `.account.save` time; display via `cols::+host` and `cols::+role` | ✅ |
| 030 | [Account Billing Renewal Override](030_account_renewal_override.md) | `.account.renewal` command; `_renewal_at` field in `{name}.json`; exact `~Renews` and `→ Next` columns in `.usage` | ✅ |
| 031 | [Account Inspect](031_account_inspect.md) | `.account.inspect` command; live endpoints 001/002/005; all memberships with selection-priority indicator | ✅ |
| 032 | [Account Marker Assignment](032_account_assign.md) | `.account.assign` command; marker-only write; no ownership stamp; live usage block when called without `name::` | ✅ |
| 033 | [Quota Cache Fallback](033_quota_cache.md) | Persist last-known quota in `{name}.json`; display cached values with staleness indicator when API unavailable | ✅ |
| 034 | [Explicit Session Model Override](034_explicit_session_model_override.md) | `set_model::` parameter on `.account.use` and `.usage`; writes explicit model ID to `~/.claude/settings.json` via `set_session_model()`, bypassing auto-override | ✅ |
| 035 | [Dedicated Model Get/Set Command](035_model_command.md) | `.model` command for standalone get/set of session model in `settings.json`; `set::` parameter; `get_session_model()` helper; no-duplication via shared `map_model_shorthand()` | ✅ |
| 036 | [Account Ownership](036_account_ownership.md) | `owner` field in `{name}.json`; ownership-neutral `.account.save` (`account_save_routine()` passes `owner: None`); `.accounts owner::0 name::X` releases ownership via `write_owner()` (Feature 064); `.accounts assignee::USER@MACHINE` is marker-only (Feature 065); nine enforcement gates (G1, G1b, G2–G8) preventing non-owner credential operations; cache-as-primary for non-owned fetch | ✅ |
| 037 | [Accounts/Usage Param Unification](037_accounts_usage_param_unification.md) | Unify `.accounts` and `.usage` to 32 shared params with different defaults; absorb `.account.unclaim` as `unclaim::` and `.account.assign` as `assign::`/`for::` params; replace 15 field toggles with `cols::`; reduce commands 18→16 | ✅ |
| 038 | [Usage Strategy Rotate](038_usage_strategy_rotate.md) | `rotate::1` on `.usage`: switch to footer-recommended account after quota table render; G5 ownership gate; dry-run preview; touch reuse from in-memory quota; deprecates `.account.rotate` | ✅ |
| 039 | [Decision Algorithm Reference](039_decision_algorithms.md) | Unified reference for 6 core decision algorithms: touch model, session model override, quota status groups, eligibility gates, next-account positive selection, quota approximation | ✅ |
| 040 | [Quota Measurement History](040_quota_measurement_history.md) | Store up to 10 real server measurements; quadratic LS approximation when API unavailable | ✅ |
| 061 | [Solo Token Conservation](061_solo_token_conservation.md) | Restrict live operations to current+owned account; others use `approximate_quota()` | ✅ |
| 062 | [Unified Session Config Recommendation](062_unified_session_config.md) | `recommended_model()` canonical function; footer Next line shows effort; rotation writes model+effort for winner | ✅ |
| 063 | [Explicit Ownership Claim](063_explicit_ownership_claim.md) | `owner::` parameter — CLI-exposed write path for account ownership; G8 gate; `force::` bypass | ✅ |
| 064 | [Active Marker and Owner Param Redesign](064_active_marker_and_owner_redesign.md) | Repurpose `active::` as `Kind::String` assign/unassign param; `owner::0` sentinel for ownership release; batch `name::X,Y,Z`; remove `assign::1`, `for::`, `unclaim::1` | ✅ |
| 065 | [Assignee Param Redesign](065_assignee_param_redesign.md) | Rename `active::` → `assignee::`; `assignee::0` sentinel = current machine (`$USER@$HOSTNAME`); `active::` REMOVED_TOGGLE | ✅ |
| 066 | [Dual-Source OAuth Quota Parsing](066_dual_source_quota_parsing.md) | Maintain correct per-model quota data when Anthropic API format changes by parsing both named-field and `limits`-array formats | ✅ |
| 067 | [Trace Timestamp Prefix](067_trace_timestamps.md) | Replace `[trace]` prefix with UTC timestamp on all diagnostic trace output; enables watchdog log correlation | ✅ |
| 068 | [Models List Command](068_models_list_command.md) | `.models` command: list available Claude models via live API or static offline catalog; `name::` filter; `format::` output | ✅ |
| 069 | [Model Select Command](069_model_select_command.md) | `.model.select` command: get/set/reset subprocess model preference in `~/.clr/config.toml` for `clr run/ask/isolated/refresh` | ✅ |
| 070 | [Account Claim and Reservation Control](070_account_claim_and_reservation_control.md) | `claim_lock` (Gate 9 unconditional exclusion + G9 bypassable explicit-command gate) and `reserve` (leading sort key, soft deprioritization) account properties; `lock::`/`reserve::` params | ✅ |

### Organization

Feature IDs 041–060 are unassigned. Features 061–069 were allocated as a separate series before the 041–060 range was needed; those IDs remain reserved.
