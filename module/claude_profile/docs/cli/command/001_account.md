# Commands: Account

Account management commands: list, save, use, delete, limits, relogin, and tag.

---

### Command: 3. `.accounts`

List all saved accounts (identity view) or run per-account mutations (`assignee::USER@MACHINE`, `owner::0`, `owner::USER@MACHINE`, `lock::0`/`lock::1`, `reserve::0`/`reserve::1`). Without `name::`: shows all accounts; with `name::EMAIL`: shows that account only; with `tags::a,b` ([feature/075](../../feature/075_account_tags.md)): only accounts carrying **all** listed tags. Column visibility controlled via `cols::` (modifies from default identity set: Account, Owner, Active, Current, Sub, Tier, Expires, Email, Provider). When data-source params are active (`refresh::1`, `touch::1`), fetches live quota using the same pipeline as `.usage` — defaults to local-only read with no HTTP fetch.

-- **Parameters:** [`name::`](../param/001_name.md) *(optional)*, [`cols::`](../param/033_cols.md), [`tags::`](../param/082_tags.md), [`assignee::`](../param/063_assignee.md), [`owner::`](../param/062_owner.md), [`lock::`](../param/067_lock.md), [`reserve::`](../param/068_reserve.md), [`force::`](../param/058_force.md), [`dry::`](../param/004_dry.md), [`set_model::`](../param/054_set_model.md), [`refresh::`](../param/019_refresh.md), [`touch::`](../param/034_touch.md), [`imodel::`](../param/035_imodel.md), [`effort::`](../param/036_effort.md), [`sort::`](../param/025_sort.md), [`desc::`](../param/026_desc.md), [`prefer::`](../param/027_prefer.md), [`count::`](../param/037_count.md), [`offset::`](../param/038_offset.md), [`only_active::`](../param/039_only_active.md), [`only_next::`](../param/040_only_next.md), [`min_5h::`](../param/041_min_5h.md), [`min_7d::`](../param/042_min_7d.md), [`only_valid::`](../param/043_only_valid.md), [`exclude_exhausted::`](../param/044_exclude_exhausted.md), [`get::`](../param/045_get.md), [`no_color::`](../param/047_no_color.md), [`live::`](../param/020_live.md), [`interval::`](../param/021_interval.md), [`jitter::`](../param/022_jitter.md), [`format::`](../param/002_format.md), [`trace::`](../param/023_trace.md)
-- **Exit:** 0 (success) | 1 (usage: invalid `name::` chars, legacy field-toggle param used, unknown `cols::` id, invalid `tags::` item, REMOVED_TOGGLE param used (`assign::`, `for::`, `unclaim::`, `active::`) — exits 1 with migration message, G8 ownership violation on `owner::0` or `owner::USER@MACHINE`, G9 claim-lock violation on `assignee::` target-side) | 2 (runtime: account not found or credential store unreadable)

**Syntax:**

```bash
clp .accounts
clp .accounts name::alice@acme.com
clp .accounts alice@acme.com                         # positional: bare name at any position
clp .accounts car                                     # prefix: first saved account starting with "car"
clp .accounts cols::+host,-tier                      # add host column, remove tier column
clp .accounts cols::-owner                            # hide owner column
clp .accounts cols::-inference_provider               # hide inference provider column
clp .accounts cols::+backend                          # show backend column (anthropic/redirect)
clp .accounts tags::kimi_pool                         # only accounts carrying kimi_pool
clp .accounts tags::kimi_pool,ci cols::+tags          # all listed tags required; show Tags column
clp .accounts assignee::user1@w003 name::alice@acme.com  # write per-machine marker for alice
clp .accounts assignee::0 name::alice@acme.com           # write marker for current machine
clp .accounts assignee::user1@w003                       # unassign (clear) marker for user1@w003
clp .accounts assignee::0                                # unassign current machine's marker
clp .accounts owner::0 name::alice@acme.com            # clear ownership (G8 gate)
clp .accounts owner::0 name::alice@acme.com force::1   # bypass G8
clp .accounts owner::user1@w003 name::alice@acme.com   # set ownership
clp .accounts lock::1 name::alice@acme.com             # claim-lock: block auto-selection and .account.use/assignee::
clp .accounts lock::0 name::alice@acme.com             # clear claim-lock
clp .accounts reserve::1 name::alice@acme.com          # deprioritize for rotation (soft — still selectable as last resort)
clp .accounts reserve::0 name::alice@acme.com          # clear reservation
clp .accounts refresh::1                              # fetch live quota (HTTP)
clp .accounts refresh::1 sort::renew                 # sorted by renewal, live data
clp .accounts format::json
clp .accounts format::table
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/001_account_name.md) | *(omit to list all)* | Show or operate on a single named account; prefix resolution supported |
| `cols::` | `string` | `""` | Column visibility modifiers: comma-separated `+col_id` / `-col_id` relative to identity default set (`account`, `owner`, `active`, `current`, `sub`, `tier`, `expires`, `email`, `inference_provider`); opt-in: `display_name`, `host`, `role`, `billing`, `model`, `uuid`, `capabilities`, `org_uuid`, `org_name`, `backend`, `tags` |
| `tags::` | `string` | *(omit)* | Subset row filter: show only accounts whose tag set contains **all** listed tags (comma-separated) — [feature/075](../../feature/075_account_tags.md) |
| `assignee::` | `string` (`USER@MACHINE` or `0`) | *(omit)* | When `name::` present: write per-machine marker `_active_{machine}_{user}` = `{name}`. When `name::` absent: clear marker for the given identity. Value `"0"` expands to `$USER@$HOSTNAME` (current machine). Value sanitized per `active_marker_filename()` rules (Feature 065; renamed from `active::`) |
| `owner::` | `string` | *(omit)* | `owner::0`: clear ownership via `write_owner(name, store, "")`; G8 gate runs before write even when `dry::1`; when `name::` absent, batch-clears all owned accounts in filtered set. `owner::USER@MACHINE`: set owner; G8 gate; `name::` required (comma-list `X,Y,Z` supported). Feature 063/064. |
| `lock::` | `bool` | `0` | Set/clear `claim_lock` in `{name}.json`; ungated write (no ownership check); comma-list `name::X,Y,Z` batch; absent `name::` batch-applies to filtered set. Gates Gate 9 (eligibility, unconditional) and G9 (`.account.use`/`assignee::` target-side, `force::1`-bypassable). Feature 070. |
| `reserve::` | `bool` | `0` | Set/clear `reserve` in `{name}.json`; ungated write (no ownership check); comma-list `name::X,Y,Z` batch; absent `name::` batch-applies to filtered set. Leading sort key in `find_next_for_strategy()` — deprioritizes without excluding. Feature 070. |
| `force::` | `bool` | `0` | Bypass G8 ownership gate on `owner::0`/`owner::USER@MACHINE` and G9 claim-lock gate on `assignee::` target-side; allows any identity to modify ownership or override a lock; no effect on `lock::`/`reserve::` writes themselves (ungated) |
| `dry::` | `bool` | `0` | Preview mutations without writing; G8 gate still runs on `owner::0 name::X dry::1` (Feature 064) |
| `set_model::` | `enum` | *(omit)* | Write session model to `settings.json`: `opus`, `sonnet`, `haiku`, `default` |
| `refresh::` | `bool` | **`0`** | Attempt OAuth token refresh via subprocess (default `0`; differs from `.usage` default of `1`) |
| `touch::` | `bool` | **`0`** | Activate idle 5h session windows via subprocess (default `0`; differs from `.usage` default of `1`) |
| `imodel::` | `enum` | `auto` | Subprocess model: `auto`, `sonnet`, `opus`, `haiku`, `keep` |
| `effort::` | `enum` | `auto` | Subprocess effort: `auto`, `low`, `normal`, `high`, `max` |
| `sort::` | `enum` | **`name`** | Row ordering and footer recommendation: `name` (default for `.accounts`), `renew`, `renews` |
| `desc::` | `bool` | `0` | Sort direction: 0 = ascending, 1 = descending |
| `prefer::` | `enum` | `any` | Weekly quota column for sort heuristics: `any`, `opus`, `sonnet` |
| `count::` | `u64` | `0` | Max rows to display (0 = all) |
| `offset::` | `u64` | `0` | Skip first N rows |
| `only_active::` | `bool` | `0` | Show only the per-machine active account |
| `only_next::` | `bool` | `0` | Show only the recommended next account |
| `min_5h::` | `f64` | `0` | Hide accounts with `5h Left` below this percentage |
| `min_7d::` | `f64` | `0` | Hide accounts with `7d Left` below this percentage |
| `only_valid::` | `bool` | `0` | Hide 🔴 (invalid/expired) rows |
| `exclude_exhausted::` | `bool` | `0` | Hide 🟡 and 🔴 rows |
| `get::` | `string` | `""` | Extract bare field value for first row |
| `no_color::` | `bool` | `0` | Strip emoji and ANSI colors |
| `live::` | `bool` | `0` | Continuous monitor mode |
| `interval::` | `u64` | `30` | Seconds between live refresh cycles (≥ 30) |
| `jitter::` | `u64` | `0` | Max random seconds added to interval |
| `format::` | [`OutputFormat`](../type/002_output_format.md) | `text` | Output format: `text`, `json`, `table` |
| `trace::` | `bool` | `0` | Print timestamped diagnostic lines to stderr |

**Algorithm (5 steps):**
1. Resolve credential store; graceful degradation on unavailability (returns `(no accounts configured)` with exit 0)
2. List all accounts; resolve and filter by `name::` when provided
3. **Mutation dispatch:** `assignee::USER@MACHINE name::X` (or `assignee::0 name::X` for current machine) → G9 claim-lock check on target `X` (unless `force::1`) then write per-machine marker; `assignee::` (no `name::`) → clear per-machine marker (no G9 check — nothing to target); `owner::0` → G8 gate then write `owner: ""` (or batch-clear all owned when `name::` absent); `owner::USER@MACHINE` → G8 gate then write owner identity (comma-list `name::` supported); `lock::0`/`lock::1` → write `claim_lock` (ungated; comma-list or batch when `name::` absent); `reserve::0`/`reserve::1` → write `reserve` (ungated; comma-list or batch when `name::` absent); REMOVED_TOGGLE param present (`assign::`, `for::`, `unclaim::`, `active::`) → exit 1 with migration message; legacy field-toggle param present → exit 1 with `cols::` migration hint
4. Parse `cols::` modifiers; read `owner` from `{name}.json` per account (when `cols.owner`); detect current account via token comparison (when `cols.current`)
5. Apply sort/filter; render in `format::`

**Examples:**

```bash
clp .accounts
# alice@acme.com
#   Owner:   user1@w003
#   Active:  yes
#   Current: no
#   Sub:     max
#   Tier:    default_claude_max_20x
#   Expires: in 2h 11m
#   Email:   alice@acme.com
#   Provider: anthropic

clp .accounts format::table
# Accounts
#
#    Account         Owner        Active  Sub   Tier                     Expires
# -  --------------  -----------  ------  ----  -----------------------  ---------
# ✓  alice@acme.com  user1@w003   yes     max   default_claude_max_20x   in 2h 11m

clp .accounts owner::0 name::alice@acme.com
# unclaimed alice@acme.com

clp .accounts assignee::bob@laptop name::alice@acme.com
# assigned alice@acme.com for bob@laptop  →  _active_laptop_bob
```

**Notes:**
- `cols::` replaces the 14 former field-toggle params (`current::`, `sub::`, `tier::`, `expires::`, `email::`, `display_name::`, `host::`, `role::`, `billing::`, `model::`, `uuid::`, `capabilities::`, `org_uuid::`, `org_name::`). Using any exits 1 with a `cols::` migration hint.
- Owner column is in the identity default set — shows `USER@MACHINE` when owned, `—` when unowned. Hide with `cols::-owner`.
- `format::json` always includes all fields regardless of `cols::`.
- `format::table` columns: flag, Account, Owner (when enabled), Active, Sub, Tier, Expires.
- Data-source params (`refresh::`, `touch::`) default to `0` — `.accounts` is local-only by default; set to `1` to activate the same live pipeline as `.usage`.
- `assignee::`, `owner::`, `lock::`, and `reserve::` are also available on `.usage` (same behavior, unified param set). `assign::`, `unclaim::`, `for::`, and `active::` are REMOVED_TOGGLE params — any invocation exits 1 with a migration message. See [Feature 065](../../feature/065_assignee_param_redesign.md) and [Feature 064](../../feature/064_active_marker_and_owner_redesign.md).
- G8 ownership gate evaluates BEFORE `dry::1` on `owner::0 name::X` (Feature 064) — a non-owner gets exit 1 even in dry-run mode.
- `current::` field (in text mode) shows `Current: yes` for the account whose `accessToken` matches `~/.claude/.credentials.json`. See [feature/016_current_account_awareness.md](../../feature/016_current_account_awareness.md).
- `lock::` and `reserve::` writes are ungated (no ownership check) — any caller may lock/unlock or reserve/unreserve any account. `claim_lock` gates selection elsewhere (Gate 9 in `find_next_for_strategy()`, unconditional; G9 on `.account.use`/`assignee::` target-side, `force::1`-bypassable); `reserve` only reorders (leading sort key), never excludes. See [Feature 070](../../feature/070_account_claim_and_reservation_control.md).
- `cols::+backend` shows each account's `backend` field (`anthropic`/`redirect`); `format::json` always includes `backend` regardless of `cols::`. See [Feature 071](../../feature/071_redirect_backend_accounts.md).
- `inference_provider` is in the identity default set — shows the account's tagged inference provider (`anthropic` when never explicitly tagged via `.account.save inference_provider::`). Hide with `cols::-inference_provider`. Distinct from `backend`: `inference_provider` groups accounts for rotation eligibility (see [algorithm/004](../../algorithm/004_eligibility_gates.md) Gate 10); `backend` selects the credential/routing mechanism (Feature 071). See [Feature 072](../../feature/072_inference_provider_selection.md).
- [Feature 075](../../feature/075_account_tags.md): `tags::a,b` filters rows to accounts whose tag set contains all listed tags (an invalid tag item exits 1); text mode adds a `Tags:` line (comma-joined, sorted) only for accounts carrying ≥1 tag; `format::json` always includes the `tags` array; `cols::+tags` adds an opt-in Tags column.

**Help Rendering Scheme:**

`.accounts.help` renders the 32 parameters above as 6 presentation groups — a command-specific rendering taxonomy, distinct from the 4 `param_group/` cross-command semantic groups referenced below (see [pattern/001_grouped_help_rendering.md](../../pattern/001_grouped_help_rendering.md) for why these differ):

| Group | Parameters |
|-------|-----------|
| Core | `name::`, `format::`, `dry::` |
| Account Ownership | `owner::`, `assignee::`, `lock::`, `reserve::`, `force::` |
| Sort Control | `sort::`, `desc::`, `prefer::` |
| Row Filtering & Pagination | `cols::`, `tags::`, `count::`, `offset::`, `only_active::`, `only_next::`, `only_valid::`, `exclude_exhausted::`, `min_5h::`, `min_7d::` |
| Display Rendering | `no_color::`, `get::` |
| Refresh & Subprocess Control | `trace::`, `refresh::`, `touch::`, `imodel::`, `effort::`, `set_model::`, `live::`, `interval::`, `jitter::` |

Each group header renders bold/colored with no bracket punctuation on a TTY, falling back to a single trailing colon (e.g. `Core:`) in plain text. Every boolean parameter's signature is shown bare (`dry::0`, never `dry::0|1`); accepted values and the default are stated once in a blanket line rather than per row. Enum-valued parameters (`imodel::`, `effort::`, `set_model::`, `format::`, `sort::`, `prefer::`) show an uppercase placeholder in the signature (e.g. `imodel::MODEL`) with actual values spelled out in the description column. The name / `::` / value signature sub-columns are independently padded so the `::` delimiter aligns vertically across all 32 rows. No version banner and no information about REMOVED parameters appear in `.accounts.help` output — the REMOVED_TOGGLE stubs (`assign::`, `for::`, `unclaim::`, `active::`) keep their existing runtime redirect-error behavior (see Notes above); they are simply invisible from `.help` text. Full rationale and general rendering rules: [pattern/001_grouped_help_rendering.md](../../pattern/001_grouped_help_rendering.md).

### Referenced Command Group

Evaluated against `.usage` under the strict [command_group](../command_group/readme.md) identity test (same dispatch function, same parameter set) — does not qualify in substance, though criterion (b) is technically satisfied at the literal-registration level: both `.accounts` and `.usage` are registered to the same function, `accounts_view_routine()` (`src/commands/accounts.rs:402`), a thin shim that branches on command name and delegates to `accounts_routine()` (`src/commands/accounts.rs:70`) for `.accounts` or `usage_routine()` (`src/usage/api.rs:78`) for `.usage` — the two branches share no logic beyond that check, and `accounts_routine()` itself has zero cross-calls with `usage_routine()`. The "same live pipeline as `.usage`" claim in the command summary above and in the Notes (`refresh::`/`touch::` "activate the same live pipeline as `.usage`") describes intended parity, not implemented behavior: `accounts_routine()` registers `refresh::`, `touch::`, `imodel::`, and `effort::` as accepted parameters (`src/registry.rs:92-95`) but never reads any of the four anywhere in its body — `grep -n "refresh\|touch\|fetch" src/commands/accounts.rs` returns no matches — and never calls `fetch_quota_for_list()` (`src/usage/fetch.rs:63`), `apply_refresh()` (`src/usage/refresh.rs:76`), or `apply_touch()` (`src/usage/touch.rs:133`), all of which are called only from `usage_routine()`. The `assignee::`/`owner::`/`lock::`/`reserve::` mutation sharing (line above) is a real shared-helper case, already covered by the `.accounts`/`.usage` rows in [`command_group/readme.md`](../command_group/readme.md) Evaluated, Not Qualifying — see that table for the full citation-backed analysis of both this claim and the mutation-dispatch one.

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`name::`](../param/001_name.md) | Account identifier or prefix |
| 2 | [`cols::`](../param/033_cols.md) | Column visibility modifiers |
| 3 | [`assignee::`](../param/063_assignee.md) | Write per-machine active-account marker |
| 4 | [`owner::`](../param/062_owner.md) | Set or release account ownership |
| 5 | [`lock::`](../param/067_lock.md) | Set or clear `claim_lock` |
| 6 | [`reserve::`](../param/068_reserve.md) | Set or clear `reserve` |
| 7 | [`force::`](../param/058_force.md) | Bypass G8 ownership gate and G9 claim-lock gate |
| 8 | [`dry::`](../param/004_dry.md) | Preview mutation without writing |
| 9 | [`set_model::`](../param/054_set_model.md) | Write session model after operation |
| 10 | [`refresh::`](../param/019_refresh.md) | Force token refresh |
| 11 | [`touch::`](../param/034_touch.md) | Activate idle 5h session window |
| 12 | [`imodel::`](../param/035_imodel.md) | Model for post-switch subprocess |
| 13 | [`effort::`](../param/036_effort.md) | Effort for post-switch subprocess |
| 14 | [`sort::`](../param/025_sort.md) | Row ordering strategy |
| 15 | [`desc::`](../param/026_desc.md) | Reverse sort direction |
| 16 | [`prefer::`](../param/027_prefer.md) | Tiebreaker sort strategy |
| 17 | [`count::`](../param/037_count.md) | Limit row count after filtering |
| 18 | [`offset::`](../param/038_offset.md) | Skip first N rows |
| 19 | [`only_active::`](../param/039_only_active.md) | Keep only active account row |
| 20 | [`only_next::`](../param/040_only_next.md) | Keep only recommended next account row |
| 21 | [`min_5h::`](../param/041_min_5h.md) | Keep rows with 5h quota ≥ N% |
| 22 | [`min_7d::`](../param/042_min_7d.md) | Keep rows with 7d quota ≥ N% |
| 23 | [`only_valid::`](../param/043_only_valid.md) | Keep non-exhausted non-expired rows |
| 24 | [`exclude_exhausted::`](../param/044_exclude_exhausted.md) | Remove exhausted rows |
| 25 | [`get::`](../param/045_get.md) | Extract bare field value from first row |
| 26 | [`no_color::`](../param/047_no_color.md) | Strip emoji and ANSI sequences |
| 27 | [`live::`](../param/020_live.md) | Continuous monitor mode |
| 28 | [`interval::`](../param/021_interval.md) | Seconds between live refresh cycles |
| 29 | [`jitter::`](../param/022_jitter.md) | Random jitter added to interval |
| 30 | [`format::`](../param/002_format.md) | Output serialization format |
| 31 | [`trace::`](../param/023_trace.md) | Diagnostic trace output |
| 32 | [`tags::`](../param/082_tags.md) | Tag subset row filter |

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Account List](../../feature/003_account_list.md) | Account enumeration and per-account block rendering |
| 2 | [Rich Account Metadata](../../feature/014_rich_account_metadata.md) | Extended metadata fields |
| 3 | [Name Shortcut Syntax](../../feature/015_name_shortcut_syntax.md) | Prefix and positional `name::` resolution |
| 4 | [Current Account Awareness](../../feature/016_current_account_awareness.md) | Token-based current account detection (`cols.current`) |
| 5 | [Extended Snapshot Fields](../../feature/021_extended_snapshot_fields.md) | Opt-in snapshot fields via `cols::+uuid` / `+capabilities` |
| 6 | [Org Identity Snapshot](../../feature/022_org_identity_snapshot.md) | Org fields via `cols::+org_uuid` / `+org_name` |
| 7 | [Host Metadata](../../feature/029_account_host_metadata.md) | `cols::+host` / `+role` from saved snapshot |
| 8 | [Account Ownership](../../feature/036_account_ownership.md) | G8 gate for `owner::0` and `owner::USER@MACHINE`; `force::` bypass |
| 9 | [Accounts/Usage Param Unification](../../feature/037_accounts_usage_param_unification.md) | 32-param unified interface; `cols::` replacing field toggles; mutation params |
| 10 | [Active Marker and Owner Param Redesign](../../feature/064_active_marker_and_owner_redesign.md) | `active::` introduced as `Kind::String` mutation param (superseded by Feature 065); `owner::0` sentinel; REMOVED_TOGGLE stubs |
| 11 | [Assignee Param Redesign](../../feature/065_assignee_param_redesign.md) | `assignee::` rename from `active::`; `assignee::0` current-machine sentinel; `active::` REMOVED_TOGGLE |
| 12 | [Account Claim And Reservation Control](../../feature/070_account_claim_and_reservation_control.md) | `lock::`/`reserve::` mutation params; Gate 9 and G9 `claim_lock` gates; `reserve` leading sort key |
| 13 | [Redirect Backend Accounts](../../feature/071_redirect_backend_accounts.md) | `cols::+backend` column showing `anthropic`/`redirect` per account |
| 14 | [Inference Provider Selection](../../feature/072_inference_provider_selection.md) | `inference_provider` default identity column; Gate 10 rotation constraint |
| 15 | [Account Tags](../../feature/075_account_tags.md) | `tags::` subset filter; `Tags:` line; `cols::+tags` column |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Rotation](../user_story/001_account_rotation.md) | Inspect accounts before and after rotation |
| 2 | [Account Onboarding](../user_story/002_onboarding.md) | Verify saved account metadata during onboarding |
| 3 | [Scripted Pipeline Automation](../user_story/004_scripted_automation.md) | Enumerate all accounts as JSON for pipeline consumption |

### Referenced Parameter Groups

| # | Group | Parameters Used |
|---|-------|-----------------|
| 1 | [Output Control](../param_group/001_output_control.md) | `format::`, `get::` |
| 2 | [Fetch Behavior](../param_group/003_fetch_behavior.md) | `refresh::`, `live::`, `interval::`, `jitter::`, `trace::`, `touch::`, `imodel::`, `effort::` |
| 3 | [Sort Control](../param_group/004_sort_control.md) | `sort::`, `desc::`, `prefer::` |
| 4 | [Display Control](../param_group/005_display_control.md) | `cols::`, `count::`, `offset::`, `only_active::`, `only_next::`, `min_5h::`, `min_7d::`, `only_valid::`, `exclude_exhausted::`, `no_color::` |

### Referenced Formats

| # | Format | Trigger |
|---|--------|---------|
| 1 | [text](../format/001_text.md) | `format::text` (default) |
| 2 | [json](../format/002_json.md) | `format::json` |
| 3 | [table](../format/003_table.md) | `format::table` |

### Referenced Patterns

| # | Pattern | Role |
|---|---------|------|
| 1 | [Grouped Column-Aligned Help Rendering](../../pattern/001_grouped_help_rendering.md) | `.accounts.help` rendering scheme (6 presentation groups, `::` alignment) |

---

### Command: 4. `.account.save`

Copies `~/.claude/.credentials.json` to `{credential_store}/{name}.credentials.json` and merges identity, model, roles, and profile metadata into the unified `{name}.json`. Machine-global state (`commands.*`, `mcpServers`, `projects`) is not captured. Use this to preserve account identity before switching. When `backend::redirect`, a different write path applies instead: no OAuth capture — `{name}.credentials.json` is written directly from `api_key::`, and `base_url`/`redirect_model` are stored in `{name}.json` alongside `backend` (see [feature/071](../../feature/071_redirect_backend_accounts.md)). `preset::kimi` pre-fills `backend::`/`base_url::`/`inference_provider::` for a Moonshot Kimi redirect account, so only `name::`, `api_key::`, and `redirect_model::` need to be given explicitly (see [feature/073](../../feature/073_kimi_provider_preset.md)).

-- **Parameters:** [`name::`](../param/001_name.md), [`dry::`](../param/004_dry.md), [`host::`](../param/048_host.md), [`tags::`](../param/082_tags.md) *(replaces the REMOVED `role::`)*, [`inference_provider::`](../param/073_inference_provider.md), [`trace::`](../param/023_trace.md), [`backend::`](../param/069_backend.md), [`preset::`](../param/074_preset.md), [`base_url::`](../param/070_base_url.md), [`api_key::`](../param/071_api_key.md), [`redirect_model::`](../param/072_redirect_model.md)
-- **Exit:** 0 (success) | 1 (usage: invalid name or no active account set; empty `inference_provider::` value; invalid `backend::` value; unrecognized `preset::` value (only `kimi` is recognized); `backend::redirect` missing one of `base_url::`/`api_key::`/`redirect_model::`; any of those three present with `backend::anthropic` or omitted `backend::`; `role::` used (REMOVED — exit 1 with `tags::` migration message); invalid `tags::` item) | 2 (runtime: credentials unreadable)

**Syntax:**

```bash
clp .account.save                                      # infer name from oauthAccount.emailAddress in ~/.claude.json (falls back to _active_{hostname}_{user})
clp .account.save name::alice@acme.com                # explicit name
clp .account.save name::alice@acme.com dry::1
clp .account.save host::workstation                   # store host label in {name}.json
clp .account.save tags::work                          # store tag set in {name}.json
clp .account.save host::workstation tags::ci,work     # host label plus tag set
clp .account.save inference_provider::kimi            # tag account with inference provider label
clp .account.save name::kimi backend::redirect base_url::https://api.moonshot.ai/anthropic api_key::"$KIMI_API_KEY" redirect_model::kimi-k3 inference_provider::kimi
clp .account.save name::kimi preset::kimi api_key::"$KIMI_API_KEY" redirect_model::kimi-k3
                                                        # same result — preset::kimi fills backend::/base_url::/inference_provider::
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/001_account_name.md) | `auto` (inferred from `oauthAccount.emailAddress` in `~/.claude.json`; falls back to per-machine active marker — see [Feature 025](../../feature/025_per_machine_active_marker.md); exits 1 if neither source present) | Account email to save as |
| `dry::` | `bool` | `0` | Preview action without executing |
| `host::` | `string` | `""` (auto-detected hostname) | Machine/host label stored in `{name}.json` (see [feature/029](../../feature/029_account_host_metadata.md)) |
| `tags::` | `string` | *(omit — tag set unchanged)* | Comma-separated tag set stored in `{name}.json` (sorted, deduplicated); replaces the REMOVED `role::` (see [param 082](../param/082_tags.md), [param 052](../param/052_role.md)) |
| `inference_provider::` | `string` | `"anthropic"` (when omitted, field absent from `{name}.json`; `list()` treats absence as `"anthropic"`) | Inference provider label stored in `{name}.json`; non-empty when provided; governs Gate 10 rotation grouping (see [param 073](../param/073_inference_provider.md)) |
| `trace::` | `bool` | `0` | Print timestamped diagnostic lines to stderr for credential read and file write steps |
| `backend::` | [`AccountBackend`](../type/005_account_backend.md) | `anthropic` | Selects OAuth capture (`anthropic`) or the static-credential redirect path (`redirect`); see [param 069](../param/069_backend.md) |
| `preset::` | `string` | *(omit)* | Named provider preset pre-filling `backend::`/`base_url::`/`inference_provider::` when omitted; only `kimi` recognized; see [param 074](../param/074_preset.md) |
| `base_url::` | `string` | *(omit; required when `backend::redirect`)* | Redirect target's API base URL; see [param 070](../param/070_base_url.md) |
| `api_key::` | `string` | *(omit; required when `backend::redirect`)* | Redirect target's static API key; see [param 071](../param/071_api_key.md) |
| `redirect_model::` | `string` | *(omit; required when `backend::redirect`)* | Redirect target's own model identifier; see [param 072](../param/072_redirect_model.md) |

**Preset resolution (before step 1):** `preset::kimi` is resolved first, before any other parameter validation. An unrecognized `preset::` value (anything but `kimi`) exits 1 immediately. Otherwise: `backend` defaults to `redirect` when `backend::` was omitted; then, using that resolved `backend` value, `base_url` defaults to `https://api.moonshot.ai/anthropic` and `inference_provider` defaults to `kimi` — each only when the corresponding parameter was omitted AND the resolved `backend` is `redirect`. Explicit `backend::`/`base_url::`/`inference_provider::` values always take precedence over these defaults. See [feature/073](../../feature/073_kimi_provider_preset.md).

**Algorithm (5 steps for `backend::anthropic`; redirect branch below for `backend::redirect`):**
1. Resolve `name::`: read `oauthAccount.emailAddress` from `~/.claude.json`; fall back to `_active_{hostname}_{user}` marker; exit 1 if neither present
2. `(when dry::0)` Copy `~/.claude/.credentials.json` → `{name}.credentials.json` (atomic write)
3. `(when dry::0)` Read `~/.claude.json` + `~/.claude/settings.json` + call `GET /api/oauth/claude_cli/roles` (best-effort); merge all into unified `{name}.json` (preserves `_renewal_at` and other keys)
4. `(when dry::0)` Write host, tags, and inference provider into `{name}.json`: `host::` (auto-captured `$USER@$HOSTNAME` when omitted); `tags::` via read-merge (omitted leaves the tag set unchanged; a first tag write converts a legacy non-empty `role` to a tag and removes the field, [feature/075](../../feature/075_account_tags.md)); `inference_provider::` via read-merge (field left absent when omitted — `list()` defaults absence to `"anthropic"`); `owner` field preserved unchanged via read-merge — `account_save_routine()` passes `owner: None` to `save()` (ownership-neutral)
5. `(when dry::0)` Write `_active_{hostname}_{user}` = `{name}` (per-machine active marker)

**Redirect branch** `(when backend::redirect)`: validates `base_url::`/`api_key::`/`redirect_model::` are all present (exit 1 naming any missing parameter) — note that `base_url::` may already be filled by `preset::kimi`'s default per the preset resolution above, so this check runs against the resolved value, not the raw CLI argument; `(when dry::0)` writes `{name}.credentials.json` containing only `accessToken` (from `api_key::`) — no `refreshToken`/`expiresAt` keys; writes `backend: "redirect"`, `base_url`, `redirect_model` into `{name}.json`. Steps 2–4 above do not apply (no `~/.claude/.credentials.json` capture, no endpoint 005 call, no host/tags merge — `base_url`/`redirect_model` serve as the redirect account's equivalent metadata). Step 5 (active marker) still applies unchanged. See [feature/071](../../feature/071_redirect_backend_accounts.md).

**Examples:**

```bash
clp .account.save
# saved current credentials as 'alice@acme.com'   (inferred from oauthAccount.emailAddress)

clp .account.save name::alice@acme.com dry::1
# [dry-run] would save current credentials as 'alice@acme.com'

clp .account.save host::workstation tags::ci,work     # host label plus tag set
# saved current credentials as 'alice@acme.com'   (host='workstation', tags='ci,work')

clp .account.save name::kimi backend::redirect base_url::https://api.moonshot.ai/anthropic api_key::"$KIMI_API_KEY" redirect_model::kimi-k3
# saved redirect-backend account 'kimi'   (backend='redirect', base_url='https://api.moonshot.ai/anthropic')

clp .account.save name::kimi preset::kimi api_key::"$KIMI_API_KEY" redirect_model::kimi-k3
# saved redirect-backend account 'kimi'   (backend='redirect', base_url='https://api.moonshot.ai/anthropic', inference_provider='kimi')

clp .account.save name::kimi preset::bogus api_key::"$KIMI_API_KEY" redirect_model::kimi-k3
# error: preset:: invalid value 'bogus' — valid values: kimi
```

**Notes:**
- Also writes `{credential_store}/_active_{hostname}_{user}` = `{name}` on every successful save (per-machine active marker via `active_marker_filename()`).
- Also calls endpoint 005 (`GET /api/oauth/claude_cli/roles`) and merges result into `{name}.json` (best-effort: failure is silently skipped).
- **Metadata refresh:** Re-running `.account.save` for an existing name refreshes the unified `{name}.json` and re-fetches endpoint 005 — this is the canonical way to refresh cached org identity without re-login. `{name}.json` is updated via read-merge (not full overwrite): the `oauthAccount` key is replaced but all other keys (e.g., `_renewal_at` set by `.account.renewal`) are preserved.
- **Ownership-neutral save:** `.account.save` never writes to the `owner` field — `account_save_routine()` passes `owner: None` to `save()`, preserving any existing `owner` via read-merge. Background refresh callers also pass `owner: None`. To release ownership, use `clp .accounts owner::0 name::EMAIL`. See [Feature 036](../../feature/036_account_ownership.md).
- **Redirect backend:** `backend` is fixed per save call — re-running `.account.save name::X` with a different `backend::` value rewrites the account from scratch per that backend's own path (not a partial update). Pre-existing accounts saved before Feature 071 have no `backend` key and are treated as `anthropic`. See [Feature 071](../../feature/071_redirect_backend_accounts.md).
- **Inference provider tagging:** `inference_provider::` is independent of `backend::` — a `backend::redirect` account may still carry any `inference_provider` label (e.g., `kimi`, `moonshot`) for Gate 10 rotation grouping; the two fields serve different purposes (routing/credential mechanism vs. rotation grouping). Defaults to `"anthropic"` when never explicitly tagged. See [Feature 072](../../feature/072_inference_provider_selection.md).
- **Tags:** `tags::` writes the full tag set (replace semantics at save); omitted, the stored set is untouched. `role::` is REMOVED — using it exits 1 with a migration message naming `tags::`; the first tag write to an account holding a legacy non-empty `role` converts it to a tag and removes the field. For pure metadata edits without re-capturing credentials, use [`.account.tag`](#command-25-accounttag). See [Feature 075](../../feature/075_account_tags.md).
- **Kimi provider preset:** `preset::kimi` pre-fills `backend::redirect`, `base_url::https://api.moonshot.ai/anthropic`, and `inference_provider::kimi` — but only for fields the caller omitted, and only once `backend` resolves to `redirect`; pairing `preset::kimi` with an explicit `backend::anthropic` leaves the account on the ordinary OAuth-capture path with none of the redirect-only defaults applied. `api_key::`/`redirect_model::` are never defaulted by any preset. Tagging an account `inference_provider::kimi` (whether via the preset or explicitly) also drives `switch_account()`'s 7 additional Kimi-tier `settings.json` env vars on `.account.use` — see [Feature 073](../../feature/073_kimi_provider_preset.md).

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Store Init](../../feature/001_account_store_init.md) | Credential store initialization before save |
| 2 | [Save Account](../../feature/002_account_save.md) | Core save algorithm and file layout |
| 3 | [Persistent Storage](../../feature/010_persistent_storage.md) | Unified `{name}.json` merge semantics |
| 4 | [Per-Machine Active Marker](../../feature/025_per_machine_active_marker.md) | `_active_{hostname}_{user}` marker written on save |
| 5 | [Host Metadata](../../feature/029_account_host_metadata.md) | `host::` metadata stored in `{name}.json`; the feature's `role::` half is superseded by tags (Feature 075) |
| 6 | [Account Ownership](../../feature/036_account_ownership.md) | Ownership model — `.account.save` is ownership-neutral (passes `owner: None`); `.accounts owner::0 name::X` releases ownership (Feature 064); `.accounts assignee::USER@MACHINE` is marker-only (Feature 065) |
| 7 | [Redirect Backend Accounts](../../feature/071_redirect_backend_accounts.md) | `backend::redirect` write path — static-credential accounts bypassing OAuth capture |
| 8 | [Inference Provider Selection](../../feature/072_inference_provider_selection.md) | `inference_provider::` write path — tags account for Gate 10 rotation grouping |
| 9 | [Kimi Provider Preset](../../feature/073_kimi_provider_preset.md) | `preset::kimi` convenience default-filling for `backend::`/`base_url::`/`inference_provider::` |
| 10 | [Account Tags](../../feature/075_account_tags.md) | `tags::` write path; `role::` removal and lazy migration |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Saving credentials during initial account setup |

### Referenced Parameter Groups

| # | Group | Parameters Used |
|---|-------|-----------------|
| 1 | [Account Targeting](../param_group/006_account_targeting.md) | `host::`, `tags::`, `inference_provider::` |
| 2 | [Fetch Behavior](../param_group/003_fetch_behavior.md) | `trace::` |
| 3 | [Redirect Backend Config](../param_group/007_redirect_backend_config.md) | `backend::`, `preset::`, `base_url::`, `api_key::`, `redirect_model::` |

---

### Command: 5. `.account.use`

Atomically overwrites `~/.claude/.credentials.json` with the named account's credentials (write-then-rename), updates the active marker (`_active_{hostname}_{user}`), and best-effort patches `~/.claude.json["oauthAccount"]` from the saved snapshot — preserving all machine-global keys untouched. When `touch::1` (default), fetches quota for the target account and spawns an isolated subprocess to activate its idle 5h session window if `five_hour.resets_at` is absent. Guarded by G5 (ownership) and G9 (`claim_lock`) — both bypassable via `force::1`. When the target account is `backend: redirect`, this command additionally writes `settings.json`'s `env.ANTHROPIC_BASE_URL`/`env.ANTHROPIC_AUTH_TOKEN`/`env.ANTHROPIC_MODEL` (clearing those same keys when switching back to a `backend: anthropic` account instead), and skips the quota/touch step entirely — there is no Anthropic quota to fetch for a foreign backend (see [feature/071](../../feature/071_redirect_backend_accounts.md)). When the target account additionally carries `inference_provider: "kimi"`, 7 more Kimi-tier `env.*` variables are written alongside the three above (and all 10 are cleared together on switch-away) — see [feature/073](../../feature/073_kimi_provider_preset.md).

-- **Parameters:** [`name::`](../param/001_name.md) **(required)**, [`dry::`](../param/004_dry.md), [`touch::`](../param/034_touch.md), [`refresh::`](../param/019_refresh.md), [`imodel::`](../param/035_imodel.md), [`effort::`](../param/036_effort.md), [`trace::`](../param/023_trace.md), [`set_model::`](../param/054_set_model.md), [`force::`](../param/058_force.md)
-- **Exit:** 0 (success) | 1 (usage: invalid name or invalid `imodel::`/`effort::`/`trace::`/`set_model::` value; G5 ownership violation unless `force::1`; G9 claim-lock violation unless `force::1`) | 2 (runtime: account not found or HOME unset) | 3 (account credentials expired — `touch::1` + fetch failed + `expiresAt` in the past, AND refresh failed or `refresh::0`)

**Syntax:**

```bash
clp .account.use name::alice@home.com
clp .account.use alice@home.com               # positional: bare name at any position
clp .account.use dry::1 alice@home.com        # reversed: arg order does not matter
clp .account.use car                           # prefix: first saved account starting with "car"
clp .account.use name::alice@home.com dry::1
clp .account.use name::alice@home.com touch::0
clp .account.use name::alice@home.com refresh::0
clp .account.use name::alice@home.com imodel::opus effort::max
clp .account.use name::alice@home.com trace::1
clp .account.use name::alice@home.com set_model::opus
clp .account.use name::alice@home.com set_model::default
clp .account.use name::alice@home.com force::1        # bypass G5 (ownership) and G9 (claim-lock)
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/001_account_name.md) | **(required)** | Account email to switch to |
| `dry::` | `bool` | `0` | Preview action without executing |
| `touch::` | `bool` | `1` | Activate idle 5h session window via subprocess after switch |
| `refresh::` | `bool` | `1` | Attempt OAuth token refresh when locally expired before refusing with exit 3 |
| `imodel::` | `enum` | `auto` | Model for post-switch subprocess: `auto` (haiku — sufficient for keep-alive), `sonnet`, `opus`, `haiku`, `keep` |
| `effort::` | `enum` | `auto` | Effort for post-switch subprocess: `auto` (`low` for any model; no flag for haiku/keep), `low`, `normal`, `high`, `max` |
| `trace::` | `bool` | `0` | Print timestamped `account.use` diagnostic lines to stderr: credential read, quota fetch, model resolution, subprocess dispatch |
| `set_model::` | `enum` | *(omit)* | Explicitly write session model to `settings.json`: `opus` (`claude-opus-4-8`), `sonnet` (`claude-sonnet-5`), `haiku` (`claude-haiku-4-5-20251001`), `default` (removes override); takes precedence over automatic `apply_model_override()` |
| `force::` | `bool` | `0` | Bypass G5 (ownership) unless owned or unclaimed; bypass G9 (`claim_lock`) unless clear. Each gate is bypassed independently — one `force::1` satisfies both |

**Algorithm (8 steps):**
1. Resolve `name::` via `AccountSelector`; load `{name}.credentials.json`; G5 ownership check (unless `force::1`); G9 `claim_lock` check (unless `force::1`) — either failing exits 1 before any write
2. `(when dry::0)` Atomically overwrite `~/.claude/.credentials.json` via write-then-rename
3. `(when dry::0)` Write `_active_{hostname}_{user}` = `{name}` (active marker)
4. `(when dry::0)` Best-effort patch `~/.claude.json["oauthAccount"]` from saved snapshot (preserves machine-global keys)
5. `(when dry::0)` Write or clear `settings.json`'s `env.*` keys per target `backend`: `redirect` → write `env.ANTHROPIC_BASE_URL`/`env.ANTHROPIC_AUTH_TOKEN`/`env.ANTHROPIC_MODEL` from `base_url`/`accessToken`/`redirect_model`, plus — when `inference_provider == "kimi"` — 7 more Kimi-tier vars (`ANTHROPIC_DEFAULT_OPUS_MODEL`/`_SONNET_MODEL`/`_HAIKU_MODEL`/`_FABLE_MODEL`/`CLAUDE_CODE_SUBAGENT_MODEL` mirroring `redirect_model`, `CLAUDE_CODE_EFFORT_LEVEL` fixed `"max"`, `CLAUDE_CODE_AUTO_COMPACT_WINDOW` sized by whether `redirect_model` starts with `kimi-k3`); `anthropic` → remove all 10 keys (the three base plus the 7 Kimi-tier, removing `env` entirely if it becomes empty), preserving any other `env`/`settings.json` key unchanged. See [feature/073](../../feature/073_kimi_provider_preset.md) and [schema/006](../../schema/006_settings_json.md).
6. `(when touch::1 AND target backend == anthropic)` Fetch quota via `GET /api/oauth/usage`; `(when refresh::1 + locally expired)` call `refresh_account_token()` first; evaluate idle: `five_hour.resets_at` absent → idle. Skipped entirely for `backend: redirect` targets — no Anthropic quota to fetch
7. `(when touch::1 + idle)` Resolve model+effort via `resolve_model()`/`resolve_effort()`; spawn isolated subprocess via `run_isolated()`
8. Session-model override: `(when set_model:: provided)` write requested model via `set_session_model()`; `(otherwise, when target was already active + valid quota)` write resolved model via `apply_model_override()` — a no-op for `backend: redirect` targets (see [algorithm/002](../../algorithm/002_session_model_override.md)'s redirect bypass)

**Examples:**

```bash
clp .account.use name::alice@home.com
# switched to 'alice@home.com'   (idle account: subprocess spawned to activate 5h session)

clp .account.use name::alice@home.com touch::0
# switched to 'alice@home.com'   (pure credential rotation — no subprocess)

clp .account.use name::alice@home.com dry::1
# [dry-run] would switch to 'alice@home.com'

clp .account.use name::alice@home.com trace::1
# 2026-06-25 · 16:40:04 · account.use  alice@home.com  reading /...alice@home.com.credentials.json
# 2026-06-25 · 16:40:04 · account.use  alice@home.com  reading: OK
# 2026-06-25 · 16:40:04 · account.use  alice@home.com  quota fetch: OK
# 2026-06-25 · 16:40:04 · account.use  alice@home.com  subprocess: scheduled (idle check removed)
# 2026-06-25 · 16:40:04 · account.use  alice@home.com  model: claude-opus-4-8  effort: low
# 2026-06-25 · 16:40:04 · account.use  alice@home.com  subprocess: spawned
# switched to 'alice@home.com'
```

**Notes:**
- `touch::1` (default): fetches quota for the target account; when fetch succeeds, always spawns `run_isolated(["--print", "."])` with resolved model/effort (subprocess is idempotent; Fix(BUG-285): idle check removed). Quota fetch failure checks `expiresAt` — if locally expired and `refresh::1` (default), attempts token refresh and re-probes touch context on success; exits 3 if refresh fails. If locally expired and `refresh::0`, exits 3 immediately. If `expiresAt` is absent or not yet expired, skips touch silently and the switch completes.
- `touch::0`: pure credential rotation — no quota fetch, no subprocess, no expiry check. Pre-Feature-027 behavior.
- `imodel::` and `effort::` follow the same resolution logic as `.usage` (Feature 026): `imodel::auto` always selects Haiku (sufficient for keep-alive pings); `resolve_effort()` maps Haiku and keep → no `--effort` flag, other models → `low`. See [feature/026](../../feature/026_subprocess_model_effort.md).
- `set_model::`: when provided, `set_session_model()` writes the requested model to `settings.json` last (after any `apply_post_switch_touch()` or `apply_model_override()`), ensuring it takes precedence. `default` removes the `model` key entirely.
- `trace::1` only produces output when `touch::1`; with `touch::0` there are no fetch operations to trace.
- G5 and G9 are checked before any file is written, including in `dry::1` mode — a non-owned or claim-locked target exits 1 even during a dry-run preview (mirrors G5–G8's existing dry-run interaction, see [feature/036](../../feature/036_account_ownership.md)).
- See [feature/027_account_use_post_switch_touch.md](../../feature/027_account_use_post_switch_touch.md) for full execution sequence and acceptance criteria; see [feature/070_account_claim_and_reservation_control.md](../../feature/070_account_claim_and_reservation_control.md) for G9 and `claim_lock` design.
- **Redirect backend:** exit 3 (credentials expired) never triggers for a `backend: redirect` target — there is no `expiresAt` to compare, so the expiry probe short-circuits to the `static` classification before any threshold check. See [feature/071](../../feature/071_redirect_backend_accounts.md) and [state_machine/002](../../state_machine/002_oauth_token_lifecycle.md)'s `static` state.
- **Kimi-tier env vars:** a `backend: redirect` target tagged `inference_provider: "kimi"` (whether via explicit `inference_provider::kimi` or via `preset::kimi`, see [feature/073](../../feature/073_kimi_provider_preset.md)) gets 7 additional `env.*` variables beyond the 3 above; any other redirect target (including one with no `inference_provider` tag) gets only the 3. Switching away from a kimi target clears all 10, not just the 3 base vars.

### Referenced Command Group

Evaluated against `.usage` and `.model` under the strict [command_group](../command_group/readme.md) identity test (same dispatch function, same parameter set) — does not qualify. `account_use_routine()` (`src/commands/account_ops.rs:19`) has zero cross-calls with `usage_routine()` (`src/usage/api.rs:78`) or `model_routine()` (`src/commands/model.rs:88`). The "same resolution logic" note above refers to `resolve_model()`/`resolve_effort()` (`src/usage/subprocess.rs:30,76`), lower-layer helpers reached from both commands via different intermediate wrappers (`.account.use` via `apply_post_switch_touch()`, `src/usage/api_switch.rs:361` calling `resolve_model()` at `src/usage/api_switch.rs:402`; `.usage` via `apply_refresh()` (`src/usage/refresh.rs:76`, calling `resolve_model()` at `src/usage/refresh.rs:117`) and `apply_touch()` (`src/usage/touch.rs:133`, calling `resolve_model()` at `src/usage/touch.rs:149`)); the `set_session_model()` note refers to `claude_profile_core::account::set_session_model()` (`../claude_profile_core/src/account/session_settings.rs`), called from `account_use_routine` (`src/commands/account_ops.rs:149`), `usage_routine` (`src/usage/api.rs:182`), and `model_routine` (`src/commands/model.rs:228,233`) — a shared write primitive in a different crate below the dispatch layer, not a shared dispatch function. Parameter sets diverge sharply: 9 params here vs. 35 on `.usage` vs. 6 on `.model` (`.model`'s `model::`/`effort_level::` and this command's `set_model::` are different parameters with different invocation semantics, not the same parameter under a different default). See [`command_group/readme.md`](../command_group/readme.md) Evaluated, Not Qualifying for the full analysis.

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Switch Account](../../feature/004_account_use.md) | Atomic credential rotation and active marker update |
| 2 | [Token Refresh](../../feature/017_token_refresh.md) | Pre-switch refresh on locally-expired token |
| 3 | [Session Touch](../../feature/024_session_touch.md) | Idle 5h window activation after switch |
| 4 | [Subprocess Model/Effort](../../feature/026_subprocess_model_effort.md) | Model and effort selection for post-switch subprocess |
| 5 | [Post-Switch Touch](../../feature/027_account_use_post_switch_touch.md) | Full execution sequence with touch and model override |
| 6 | [Account Ownership](../../feature/036_account_ownership.md) | G5 ownership gate; `force::1` bypass and dry-run interaction pattern |
| 7 | [Account Claim And Reservation Control](../../feature/070_account_claim_and_reservation_control.md) | G9 `claim_lock` gate; `force::1` bypass |
| 8 | [Redirect Backend Accounts](../../feature/071_redirect_backend_accounts.md) | `env.*` write/clear in `settings.json`; touch/quota and model-override skip for redirect targets |
| 9 | [Kimi Provider Preset](../../feature/073_kimi_provider_preset.md) | 7 additional Kimi-tier `env.*` vars written/cleared alongside the base 3, gated on `inference_provider == "kimi"` |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Rotation](../user_story/001_account_rotation.md) | Primary command for switching to a named account |

### Referenced Parameter Groups

| # | Group | Parameters Used |
|---|-------|-----------------|
| 1 | [Fetch Behavior](../param_group/003_fetch_behavior.md) | `trace::`, `touch::`, `imodel::`, `effort::` |

---

### Command: 6. `.account.delete`

Removes `{credential_store}/{name}.credentials.json` and `{name}.json` from the credential store, plus any legacy satellite files from pre-consolidation layout.

-- **Parameters:** [`name::`](../param/001_name.md) **(required)**, [`dry::`](../param/004_dry.md), [`trace::`](../param/023_trace.md), [`force::`](../param/058_force.md)
-- **Exit:** 0 (success) | 1 (usage: invalid name; G6 ownership violation unless `force::1`) | 2 (runtime: account not found)

**Syntax:**

```bash
clp .account.delete name::alice@oldco.com
clp .account.delete alice@oldco.com          # positional: bare name at any position
clp .account.delete dry::1 alice@oldco.com   # reversed: arg order does not matter
clp .account.delete car                      # prefix
clp .account.delete name::alice@oldco.com dry::1
clp .account.delete name::alice@oldco.com force::1   # bypass G6 ownership gate
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/001_account_name.md) | **(required)** | Account email to delete |
| `dry::` | `bool` | `0` | Preview action without executing |
| `trace::` | `bool` | `0` | Print timestamped diagnostic lines to stderr for each file removal step |
| `force::` | `bool` | `0` | Bypass G6 ownership gate; allow deleting a non-owned account |

**Algorithm (5 steps):**
1. Resolve `name::` via `AccountSelector`; validate account exists in credential store
2. G6 ownership check (unless `force::1`) — exits 1 on violation; evaluated before the `dry::1` check so dry-run still surfaces ownership violations
3. `(when dry::0)` Delete `{name}.credentials.json`
4. `(when dry::0)` Best-effort delete `{name}.json` + legacy files (`.claude.json`, `.settings.json`, `.roles.json`, `.profile.json`; skip missing)
5. `(when dry::0 + deleted account = active)` Delete `_active_{hostname}_{user}` marker

**Examples:**

```bash
clp .account.delete name::alice@oldco.com
# deleted account 'alice@oldco.com'

clp .account.delete name::alice@oldco.com dry::1
# [dry-run] would delete account 'alice@oldco.com'
```

**Notes:**
- Metadata file (`{name}.json`) and legacy satellite files are removed best-effort: missing files are silently skipped.
- Deleting the active account also removes the active marker (`_active_{hostname}_{user}`).
- G6 is checked before any file is written, including in `dry::1` mode — a non-owned target exits 1 even during a dry-run preview (mirrors G5's dry-run interaction on `.account.use`). See [feature/036_account_ownership.md](../../feature/036_account_ownership.md).

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Delete Account](../../feature/005_account_delete.md) | File removal sequence and legacy satellite cleanup |
| 2 | [Account Ownership](../../feature/036_account_ownership.md) | G6 ownership gate; `force::1` bypass |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Removing stale accounts during account management |

### Referenced Parameter Groups

| # | Group | Parameters Used |
|---|-------|-----------------|
| 1 | [Fetch Behavior](../param_group/003_fetch_behavior.md) | `trace::` |

---

### Command: 11. `.account.limits`

Show rate-limit utilization for the active or named account. Displays session (5h) usage, weekly all-model (7d) usage, and rate-limit status with percentage consumed and reset times. Rejects `backend: redirect` accounts — there is no Anthropic rate-limit data for a foreign backend.

-- **Parameters:** [`name::`](../param/001_name.md) *(optional)*, [`format::`](../param/002_format.md), [`trace::`](../param/023_trace.md)
-- **Exit:** 0 (success) | 1 (usage: invalid `name::` chars; target account is `backend: redirect`) | 2 (runtime: account not found, data unavailable, HOME unset)

**Syntax:**

```bash
clp .account.limits
clp .account.limits name::alice@acme.com
clp .account.limits alice@acme.com            # positional: bare name at any position
clp .account.limits format::json alice@acme.com  # reversed: arg order does not matter
clp .account.limits car                   # prefix
clp .account.limits format::json
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/001_account_name.md) | *(omit for active)* | Query a named account instead of the active account |
| `format::` | [`OutputFormat`](../type/002_output_format.md) | `text` | Output format |
| `trace::` | `bool` | `0` | Print timestamped diagnostic lines to stderr for credential store read and API call |

**Algorithm (3 steps):**
1. Resolve `name::` (omit → active account from `_active_{hostname}_{user}` marker); load credentials; **Anthropic-only guard:** exit 1 with an explanatory message if the resolved account is `backend: redirect` — no HTTP request is made
2. Fetch rate-limit headers via `fetch_rate_limits()` (`anthropic-ratelimit-unified-*` response headers)
3. Render session (5h), weekly all-model (7d), and weekly sonnet utilization in requested `format::`

**Examples:**

```bash
clp .account.limits
# Session (5h):   62%  resets in 1h 48m
# Weekly (all):   41%  resets in 3d 12h
# Weekly (sonnet): 38%  resets in 3d 12h

clp .account.limits format::json
# {"session_pct":62,"session_reset_secs":6480,"weekly_all_pct":41,"weekly_all_reset_secs":302400,"weekly_sonnet_pct":38,"weekly_sonnet_reset_secs":302400}

clp .account.limits name::kimi
# error: '.account.limits' is Anthropic-only — 'kimi' is a redirect-backend account (no rate-limit data available)
```

**Notes:**
- Data source: `anthropic-ratelimit-unified-*` response headers; transport: `claude_quota::fetch_rate_limits()`. See [feature/013_account_limits.md](../../feature/013_account_limits.md).
- **Redirect backend:** rejected outright (exit 1, no HTTP request) rather than approximated — a foreign backend has no Anthropic rate-limit headers to parse. See [feature/071](../../feature/071_redirect_backend_accounts.md).

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Account Limits](../../feature/013_account_limits.md) | Rate-limit header parsing and utilization rendering |
| 2 | [Redirect Backend Accounts](../../feature/071_redirect_backend_accounts.md) | Anthropic-only guard rejecting `backend: redirect` accounts |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Multi-Account Quota Monitoring](../user_story/003_quota_monitoring.md) | Per-account rate-limit utilization check |

### Referenced Parameter Groups

| # | Group | Parameters Used |
|---|-------|-----------------|
| 1 | [Output Control](../param_group/001_output_control.md) | `format::` |
| 2 | [Fetch Behavior](../param_group/003_fetch_behavior.md) | `trace::` |

### Referenced Formats

| # | Format | Trigger |
|---|--------|---------|
| 1 | [text](../format/001_text.md) | `format::text` (default) |
| 2 | [json](../format/002_json.md) | `format::json` |

---

### Command: 12. `.account.relogin`

Force browser-based re-authentication for a named account whose `refreshToken` is expired or revoked. This is the recovery path when `refresh::1` silently fails (trace shows `run_isolated: OK credentials=None` — Claude starts but performs no OAuth refresh because the refresh token itself is dead).

-- **Parameters:** [`name::`](../param/001_name.md) *(optional, defaults to active)*, [`dry::`](../param/004_dry.md), [`trace::`](../param/023_trace.md), [`force::`](../param/058_force.md)
-- **Exit:** 0 (success: credentials refreshed and saved) | 1 (usage: invalid name value; G7 ownership violation unless `force::1`) | 2 (runtime: name omitted and no active account; account not found; or Claude spawn failed) | 3 (timeout or login abandoned: claude exited without updating credentials)

**Syntax:**

```bash
clp .account.relogin                   # default: active account
clp .account.relogin name::carol@example.com
clp .account.relogin carol@example.com          # positional: bare name at any position
clp .account.relogin dry::1 carol@example.com   # reversed: arg order does not matter
clp .account.relogin car               # prefix
clp .account.relogin name::carol@example.com dry::1
clp .account.relogin dry::1            # dry-run for active account
clp .account.relogin name::carol@example.com force::1   # bypass G7 ownership gate
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/001_account_name.md) | *(active account)* | Account to re-authenticate; omit to use the currently active account |
| `dry::` | `bool` | `0` | Preview the steps without executing |
| `trace::` | `bool` | `0` | Print timestamped diagnostic lines to stderr for each step: store read, switch, spawn, credential change, save, restore |
| `force::` | `bool` | `0` | Bypass G7 ownership gate; allow re-authenticating a non-owned account |

**Algorithm (7 steps):**
1. Resolve `name::` via [`AccountSelector`](../type/004_account_selector.md) → validate account exists in credential store
2. G7 ownership check (unless `force::1`) — exits 1 on violation; evaluated before the `dry::1` check so dry-run still surfaces ownership violations
3. Snapshot the current active account name (for restoration after login)
4. `switch_account(name)` — makes the named account active in `~/.claude/`
5. Spawn `claude` with inherited TTY (stdin/stdout/stderr connected — NOT isolated subprocess) — Claude detects empty or invalid credentials and opens the browser login page
6. Wait for `claude` to exit; if `~/.claude/.credentials.json` changed → `account::save(name)` propagates fresh credentials to credential store
7. `switch_account(original_active)` — restore the prior active account

**Examples:**

```bash
clp .account.relogin dry::1
# [dry-run] would re-authenticate 'alice@example.com' via browser login

clp .account.relogin name::carol@example.com
# re-authenticated 'carol@example.com' — credentials saved

clp .account.relogin name::carol@example.com dry::1
# [dry-run] would re-authenticate 'carol@example.com' via browser login
```

**Notes:**
- Requires a TTY — `clp .account.relogin` in a piped non-TTY context will fail at step 5 (Claude cannot open a browser or display the login prompt).
- The `claude` subprocess runs with the full inherited environment; no credential isolation (contrast with `refresh::1` which uses an isolated subprocess).
- If `claude` exits without updating `~/.claude/.credentials.json`, the command exits 3. The active account is still restored (step 7 runs regardless of outcome).
- Use this when `clp .usage refresh::1 trace::1` shows `run_isolated: OK credentials=None` for an account — that trace indicates a dead refresh token requiring full browser re-auth.
- G7 is checked before any state change, including in `dry::1` mode — a non-owned target exits 1 even during a dry-run preview (mirrors G5's dry-run interaction on `.account.use`). See [feature/036_account_ownership.md](../../feature/036_account_ownership.md).

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Auto Rotate](../../feature/008_auto_rotate.md) | Relogin as recovery path for dead refresh tokens |
| 2 | [Account Ownership](../../feature/036_account_ownership.md) | G7 ownership gate; `force::1` bypass |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Re-authenticating an account with expired refresh token |

### Referenced Parameter Groups

| # | Group | Parameters Used |
|---|-------|-----------------|
| 1 | [Fetch Behavior](../param_group/003_fetch_behavior.md) | `trace::` |

---

### Command: 13. `.account.rotate` *(deprecated — Feature 038)*

**DEPRECATED** — hidden redirector; always exits 1. Use `.usage rotate::1` instead.

```bash
clp .usage rotate::1
clp .usage rotate::1 sort::renews
clp .usage rotate::1 dry::1
```

See [feature/038_usage_strategy_rotate.md](../../feature/038_usage_strategy_rotate.md) for full behavioral specification.

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Scripted Pipeline Automation](../user_story/004_scripted_automation.md) | Automated account selection in pipelines |

---

### Command: 14. `.account.renewal`

Set, preview, or clear the billing renewal timestamp override (`_renewal_at`) stored in `{name}.json`. When set, the `.usage` `~Renews` column shows an exact duration (`in Xh Ym`) instead of the estimated `~`-prefixed value derived from `org_created_at`. Supports single account, comma-separated list, or `name::all` to update every saved account in one operation.

-- **Parameters:** [`name::`](../param/001_name.md) **(required)**, [`at::`](../param/049_at.md), [`from_now::`](../param/050_from_now.md), [`clear::`](../param/051_clear.md), [`dry::`](../param/004_dry.md), [`trace::`](../param/023_trace.md)
-- **Exit:** 0 (success) | 1 (usage: no operation provided, conflicting params, or invalid format) | 2 (runtime: account not found or credential store unreadable)

**Syntax:**

```bash
clp .account.renewal name::alice@acme.com at::2026-06-29T21:00:00Z
clp .account.renewal name::alice@acme.com from_now::+1h30m
clp .account.renewal name::alice@acme.com from_now::-30m
clp .account.renewal name::alice@acme.com clear::1
clp .account.renewal name::all from_now::+0m
clp .account.renewal name::alice@acme.com,bob@acme.com at::2026-06-29T21:00:00Z
clp .account.renewal name::alice@acme.com at::2026-06-29T21:00:00Z dry::1
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/001_account_name.md) or `all` or comma-list | **(required)** | Target account(s): single email/prefix, comma-separated list, or `all` for every saved account |
| `at::` | `string` | *(omit)* | Absolute ISO-8601 UTC renewal timestamp (e.g., `2026-06-29T21:00:00Z`); mutually exclusive with `from_now::` and `clear::` |
| `from_now::` | `string` | *(omit)* | Signed duration delta from now (e.g., `+3h30m`, `-30m`, `+0m`); mutually exclusive with `at::` and `clear::` |
| `clear::` | `bool` | `0` | Remove `_renewal_at` from `{name}.json`; mutually exclusive with `at::` and `from_now::` |
| `dry::` | `bool` | `0` | Preview operation without writing files |
| `trace::` | `bool` | `0` | Print timestamped diagnostic lines to stderr for each file read and write step |

**Algorithm (4 steps):**
1. Resolve target account list from `name::`: single email/prefix, comma-separated list, or `all` (every saved account)
2. For each target: read `{name}.json`
3. Compute new `_renewal_at` value from `at::` (absolute ISO-8601), `from_now::` (signed delta), or `clear::1` (remove key)
4. `(when dry::0)` Write `{name}.json` with updated `_renewal_at` key per account

**Examples:**

```bash
clp .account.renewal name::alice@acme.com at::2026-06-29T21:00:00Z
# renewal set for 'alice@acme.com': 2026-06-29T21:00:00Z  (in 30d 14h)

clp .account.renewal name::all from_now::+0m
# renewal set for 'alice@acme.com': 2026-05-29T18:34:22Z  (now)
# renewal set for 'bob@acme.com':   2026-05-29T18:34:22Z  (now)
# renewal set for 'carol@acme.com': 2026-05-29T18:34:22Z  (now)

clp .account.renewal name::alice@acme.com clear::1
# renewal cleared for 'alice@acme.com'  (~Renews will show estimate from org_created_at)

clp .account.renewal name::alice@acme.com at::2026-06-29T21:00:00Z dry::1
# [dry-run] would set renewal for 'alice@acme.com': 2026-06-29T21:00:00Z  (in 30d 14h)
```

**Notes:**
- `_renewal_at` is stored as a top-level key in `{name}.json` alongside `oauthAccount`. It is preserved when `clp .account.save` re-saves that account (read-merge).
- Past `_renewal_at` values are auto-advanced monthly by `.usage` at render time — no need to re-set after each billing cycle.
- `from_now::+0m` sets the override to the current time, which immediately enters the monthly auto-advance cycle.
- `name::all` targets every account in the credential store at the time of execution.
- See [feature/030_account_renewal_override.md](../../feature/030_account_renewal_override.md) for full semantics, `~Renews` rendering rules, and acceptance criteria.

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Account Renewal Override](../../feature/030_account_renewal_override.md) | `_renewal_at` storage, monthly auto-advance, and `~Renews` rendering |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Set accurate billing renewal dates during account setup |

### Referenced Parameter Groups

| # | Group | Parameters Used |
|---|-------|-----------------|
| 1 | [Fetch Behavior](../param_group/003_fetch_behavior.md) | `trace::` |

---

### Command: 15. `.account.inspect`

Unified live account diagnostic — identity, subscription, org, and quota utilization for one account. Calls endpoints 002 (`GET /api/oauth/account`), 005 (`GET /api/oauth/claude_cli/roles`), and 001 (`GET /api/oauth/usage`) and renders identity fields (tagged_id, uuid, email, name), ALL membership entries with a selection-priority indicator, capabilities, rate-limit tier, and 5h/7d/Sonnet quota utilization with reset countdowns. Primary use case: diagnosing account state and remaining quota (see BUG-237 / feature 031). Rejects `backend: redirect` accounts — none of these three Anthropic endpoints have any meaning for a foreign backend.

-- **Parameters:** [`name::`](../param/001_name.md), [`refresh::`](../param/019_refresh.md), [`trace::`](../param/023_trace.md), [`format::`](../param/002_format.md)
-- **Exit:** 0 (success) | 1 (usage: invalid param; target account is `backend: redirect`) | 2 (runtime: account not found or credential store unreadable)

**Syntax:**

```bash
clp .account.inspect                    # default: active account
clp .account.inspect name::alice@acme.com
clp .account.inspect alice             # prefix
clp .account.inspect refresh::0        # skip token refresh on expired credentials
clp .account.inspect format::json
clp .account.inspect trace::1          # show timestamped diagnostic endpoint calls to stderr
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/001_account_name.md) | *(active account)* | Account to inspect; omit to use the currently active account |
| `refresh::` | `bool` | `1` | Attempt OAuth token refresh via isolated subprocess when `expiresAt` is locally expired, before endpoint calls |
| `trace::` | `bool` | `0` | Print timestamped diagnostic lines to stderr for each endpoint call: URL, HTTP status, field extraction summary |
| `format::` | [`OutputFormat`](../type/002_output_format.md) | `text` | Output format: `text` (default) or `json` |

**Algorithm (5 steps):**
1. Resolve `name::` (omit → active account from `_active_{hostname}_{user}` marker); load credentials; **Anthropic-only guard:** exit 1 with an explanatory message if the resolved account is `backend: redirect` — no endpoint call is made
2. `(when refresh::1 + locally expired)` Call `refresh_account_token()` to obtain a fresh token
3. Call endpoint 002 (`GET /api/oauth/account`), endpoint 005 (`GET /api/oauth/claude_cli/roles`), and endpoint 001 (`GET /api/oauth/usage`) — each independently; identity/org failure falls back to local snapshots with `(snapshot)` suffix per field; quota failure (transient errors — not 401/403) falls back to the local quota cache (Feature 033); quota section omitted only when both live endpoint and cache are unavailable
4. Apply membership selection priority: `billing_type=stripe_subscription + claude_max` > `billing_type=stripe_subscription` > `memberships[0]`
5. Render all fields in requested `format::`

**Output (text):**

```
Account:         alice@acme.com
Name:            Alice (Alice)
Email:           alice@acme.com
Status:          🟢 valid (expires in 3h 52m)
Tagged ID:       user_01abc...def
UUID:            aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee

Memberships:     2
  [0]  billing_type=none              has_max=false  capabilities=[chat]
  [1]  billing_type=stripe_subscription  has_max=true   capabilities=[claude_max, chat]  ← selected

Billing:         stripe_subscription
Has Max:         yes
Capabilities:    [claude_max, chat]
Tier:            default_claude_max_20x

Session (5h):    45% consumed, resets in 12m
Weekly (7d):     33% consumed, resets in 1d 5h
Sonnet (7d):     53% consumed, resets in 1d 5h

Org:             alice@acme.com's Organization
Org UUID:        aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee
Org Role:        admin
Workspace UUID:  (none)
Workspace:       (none)
```

**Membership selection priority:**

| Priority | Criteria |
|----------|----------|
| 1 (highest) | `billing_type == "stripe_subscription"` AND capabilities contain `"claude_max"` |
| 2 | `billing_type == "stripe_subscription"` (any capabilities) |
| 3 (fallback) | `memberships[0]` |

The selected membership is marked `← selected` when there are multiple memberships; the `Billing:` and `Has Max:` fields reflect the selected membership.

**Examples:**

```bash
clp .account.inspect
# Account:     alice@acme.com
# Status:      🟢 valid (expires in 3h 52m)
# ...

clp .account.inspect name::i5@wbox.pro
# Account:     i5@wbox.pro
# Memberships: 2
#   [0]  billing_type=none              has_max=false  ...
#   [1]  billing_type=stripe_subscription  has_max=true  ...  ← selected
# Billing:     stripe_subscription

clp .account.inspect format::json | jq '.memberships | length'
# 2
```

**Notes:**
- Endpoints 002, 005, and 001 are called independently. A failure on one endpoint falls back to the local snapshot from `{name}.json` with a `(snapshot)` suffix per field; quota fields (endpoint 001) fall back to the local quota cache on transient errors (not 401/403); the quota section is omitted only when both the live endpoint and the cache are unavailable.
- `refresh::1` (default) behaves identically to `.usage`'s `refresh::1`: calls `refresh_account_token()` once when `expiresAt` is locally expired; retries endpoint calls with the fresh token.
- See [feature/031_account_inspect.md](../../feature/031_account_inspect.md) for full design, graceful fallback semantics, and all acceptance criteria.
- **Redirect backend:** rejected outright (exit 1, no endpoint call) rather than approximated — same rationale as [`.account.limits`](#command-11-accountlimits). See [feature/071](../../feature/071_redirect_backend_accounts.md).

### Referenced Command Group

Evaluated against `.account.use` and `.usage` under the strict [command_group](../command_group/readme.md) identity test (same dispatch function, same parameter set) — does not qualify. `account_inspect_routine()` (`src/commands/account_inspect.rs:125`) has zero cross-calls with `account_use_routine()` (`src/commands/account_ops.rs:19`) or `usage_routine()` (`src/usage/api.rs:78`). The "behaves identically" claim above is precise only about the underlying primitive: `account_inspect_routine()` calls `attempt_expired_token_refresh()` (`src/usage/api_switch.rs:95`) at `src/commands/account_inspect.rs:176`, and `account_use_routine()` calls the same wrapper at `src/commands/account_ops.rs:215` — that specific wrapper is genuinely shared with `.account.use`, not `.usage`. `usage_routine()` instead calls a different wrapper, `apply_refresh()` (`src/usage/refresh.rs:76`, invoked at `src/usage/api.rs:157`), which iterates the full account list with 401/403/429 retry branching that `attempt_expired_token_refresh()` does not have. All three commands converge only at the deepest shared primitive, `refresh_account_token()` (`../claude_profile_core/src/account/refresh.rs`) — ordinary layered reuse of one OAuth-refresh function, not evidence of shared dispatch. Parameter sets also diverge: 4 params here vs. 9 on `.account.use` vs. 35 on `.usage`. See [`command_group/readme.md`](../command_group/readme.md) Evaluated, Not Qualifying for the full analysis.

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Account Inspect](../../feature/031_account_inspect.md) | Unified account diagnostic — identity, subscription, org, and quota utilization |
| 2 | [Redirect Backend Accounts](../../feature/071_redirect_backend_accounts.md) | Anthropic-only guard rejecting `backend: redirect` accounts |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | Live multi-endpoint inspection for subscription diagnosis |

### Referenced Parameter Groups

| # | Group | Parameters Used |
|---|-------|-----------------|
| 1 | [Output Control](../param_group/001_output_control.md) | `format::` |
| 2 | [Fetch Behavior](../param_group/003_fetch_behavior.md) | `refresh::`, `trace::` |

### Referenced Formats

| # | Format | Trigger |
|---|--------|---------|
| 1 | [text](../format/001_text.md) | `format::text` (default) |
| 2 | [json](../format/002_json.md) | `format::json` |

---

### Command: 16. `.account.assign` *(removed — Feature 037; migration path superseded — Feature 064/065)*

**Fully removed (Feature 037).** The interim `.accounts assign::1 name::X` migration path is also removed (Feature 064 — `assign::` is now a REMOVED_TOGGLE). The `active::` migration path introduced in Feature 064 is itself now a REMOVED_TOGGLE (Feature 065). Use `.accounts assignee::USER@MACHINE name::X` (or `assignee::0 name::X` for current machine) instead.

```bash
clp .accounts assignee::user1@w003 name::alice@corp.com
clp .accounts assignee::0 name::alice@corp.com          # current machine
clp .accounts assignee::bob@laptop name::alice@corp.com
clp .accounts assignee::user1@w003                      # unassign marker (no name::)
clp .accounts assignee::0                               # unassign current machine's marker
```

---

### Command: 17. `.account.unclaim` *(removed — Feature 037; migration path superseded — Feature 064)*

**Fully removed (Feature 037).** The interim `.accounts unclaim::1 name::X` migration path is also removed (Feature 064 — `unclaim::` is now a REMOVED_TOGGLE). Use `.accounts owner::0 name::X` instead.

```bash
clp .accounts owner::0 name::alice@acme.com
clp .accounts owner::0 name::alice@acme.com dry::1
clp .accounts owner::0 name::alice@acme.com force::1    # bypass G8
clp .accounts owner::0                                   # batch-clear all owned accounts
```

---

### Command: 25. `.account.tag`

Mutates a saved account's [Tag](../../type/003_tag.md) set in `{name}.json` — add, remove, or replace — without touching credentials. Tags change operationally far more often than accounts are re-saved; re-running `.account.save` re-captures live credentials as a side effect, which is wrong for a pure metadata edit. Writes are ungated (no ownership check) with comma-list `name::` batching and `dry::1` preview — the same doctrine as `lock::`/`reserve::` ([Feature 070](../../feature/070_account_claim_and_reservation_control.md)).

-- **Parameters:** [`name::`](../param/001_name.md) *(required)*, [`add::`](../param/083_add.md), [`remove::`](../param/084_remove.md), [`tags::`](../param/082_tags.md), [`dry::`](../param/004_dry.md)
-- **Exit:** 0 (success, including remove of an absent tag) | 1 (usage: missing `name::`, none of `add::`/`remove::`/`tags::` given, more than one of them given, invalid tag item) | 2 (runtime: named account not found or credential store unreadable)

**Syntax:**

```bash
clp .account.tag name::alice@acme.com add::kimi_pool,ci     # union into existing set
clp .account.tag name::alice@acme.com remove::ci            # remove (idempotent)
clp .account.tag name::alice@acme.com tags::kimi_pool       # replace whole set
clp .account.tag name::alice@acme.com,bob@acme.com add::ci  # batch
clp .account.tag name::alice@acme.com tags::ci dry::1       # preview
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/001_account_name.md) | *(required)* | Target account, or comma-list `X,Y,Z` applying the same operation to each |
| `add::` | `string` | *(omit)* | Comma-separated tags to union into the set (see [param 083](../param/083_add.md)) |
| `remove::` | `string` | *(omit)* | Comma-separated tags to remove; absent tags are a no-op success (see [param 084](../param/084_remove.md)) |
| `tags::` | `string` | *(omit)* | Comma-separated set replacing the whole tag set (see [param 082](../param/082_tags.md)) |
| `dry::` | `bool` | `0` | Preview all writes without touching disk |

**Operation dispatch** (exactly one of the three must be given):

| Params given | Effect |
|--------------|--------|
| `add::a,b` | Union `{a, b}` into the existing set (dedup, sort) |
| `remove::a` | Remove listed tags; removing an absent tag is a no-op success |
| `tags::a,b` | Replace the whole set |
| `add::` + `remove::` together | Exit 1 — one operation per invocation |
| `tags::` + (`add::` or `remove::`) | Exit 1 — replace is mutually exclusive with incremental ops |
| none of the three | Exit 1 — no operation given |

**Algorithm (6 steps):**
1. Validate exactly one of `add::`/`remove::`/`tags::` is given — zero or more than one exits 1
2. Require `name::`; split comma-list; each item must resolve to a saved account (exit 2 naming the first missing one; no partial writes)
3. Normalize the operation's tag list per [type/003](../../type/003_tag.md): lowercase, validate charset `[a-z0-9_-]` and 1–64 length, deduplicate — exit 1 naming the first offending tag
4. Per account: read `{name}.json`; when a non-empty legacy `role` field is present, convert it to a tag (lowercased, sanitized to the tag charset), merge it into the working set, and mark the `role` field for removal (lazy migration — fires on every tag write, including `remove::`)
5. Apply the operation to the working set (union / difference / replace); deduplicate and sort the result
6. `(when dry::0)` Write the result via read-merge (removing `role` when step 4 fired); print a per-account confirmation line; `dry::1` prints the would-be result instead

**Examples:**

```bash
clp .account.tag name::alice@acme.com add::kimi_pool
# alice@acme.com tags: [ci, kimi_pool]

clp .account.tag name::alice@acme.com remove::nonexistent
# alice@acme.com tags: [ci, kimi_pool]   (no-op — tag not present)

clp .account.tag name::alice@acme.com tags::personal
# alice@acme.com tags: [personal]

clp .account.tag name::alice@acme.com,bob@acme.com add::ci dry::1
# [dry-run] alice@acme.com tags: [ci, personal]
# [dry-run] bob@acme.com tags: [ci]

clp .account.tag name::alice@acme.com add::a remove::b
# exit 1: one of add::/remove::/tags:: per invocation

clp .account.tag name::alice@acme.com
# exit 1: no operation given — use add::, remove::, or tags::

clp .account.tag name::alice@acme.com add::Bad!Tag
# exit 1: invalid tag 'bad!tag'
```

**Notes:**
- Writes are **ungated** — no ownership (G8) or claim-lock check; any caller may retag any account. Tags are fleet-operations metadata, not credential operations ([Feature 070](../../feature/070_account_claim_and_reservation_control.md) doctrine).
- Never touches `{name}.credentials.json`, the active marker, or any live credential — pure `{name}.json` metadata edit (contrast [`.account.save`](#command-4-accountsave), which re-captures credentials).
- Lazy `role`→tag migration (step 4) fires on **any** of the three operations — even `remove::` — so any tag write finishes the account's migration.
- Tag semantics (charset, normalization, set behavior): [type/003](../../type/003_tag.md). Rotation filtering by these tags: [Feature 076](../../feature/076_identity_tag_filter.md) (Gate 11).

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Account Tags](../../feature/075_account_tags.md) | Owning feature — mutation semantics, lazy migration, batch/dry (AC-05…AC-10) |
| 2 | [Account Claim And Reservation Control](../../feature/070_account_claim_and_reservation_control.md) | Structural precedent — ungated metadata writes, comma-list batch, `dry::1` |
| 3 | [Identity Tag Filter](../../feature/076_identity_tag_filter.md) | Consumer — Gate 11 evaluates the tag sets this command writes |

### Referenced Types

| # | Type | Role |
|---|------|------|
| 1 | [Tag](../../type/003_tag.md) | Value contract — charset, normalization, set semantics, migration rules |
| 2 | [AccountName](../type/001_account_name.md) | `name::` value type |

### Referenced Schema

| # | Schema | Role |
|---|--------|------|
| 1 | [Account JSON](../../schema/002_account_json.md) | `tags` array written; legacy `role` field removed on migration |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Rotation](../user_story/001_account_rotation.md) | Partition the fleet into rotation pools via tags |

### Referenced Parameter Groups

| # | Group | Parameters Used |
|---|-------|-----------------|
| 1 | [Account Targeting](../param_group/006_account_targeting.md) | `tags::` (replace mode) |
