# Commands :: Account

Account management commands: list, save, use, delete, limits, and relogin.

---

### Command :: 3. `.accounts`

List all saved accounts (identity view) or run per-account mutations (`assign::1`, `unclaim::1`). Without `name::`: shows all accounts; with `name::EMAIL`: shows that account only. Column visibility controlled via `cols::` (modifies from default identity set: Account, Owner, Active, Current, Sub, Tier, Expires, Email). When data-source params are active (`refresh::1`, `touch::1`), fetches live quota using the same pipeline as `.usage` — defaults to local-only read with no HTTP fetch.

-- **Parameters:** [`name::`](../param/001_name.md) *(optional)*, [`cols::`](../param/033_cols.md), [`assign::`](../param/057_assign.md), [`unclaim::`](../param/056_unclaim.md), [`force::`](../param/058_force.md), [`for::`](../param/053_for.md), [`dry::`](../param/004_dry.md), [`set_model::`](../param/054_set_model.md), [`refresh::`](../param/019_refresh.md), [`touch::`](../param/034_touch.md), [`imodel::`](../param/035_imodel.md), [`effort::`](../param/036_effort.md), [`sort::`](../param/025_sort.md), [`desc::`](../param/026_desc.md), [`prefer::`](../param/027_prefer.md), [`next::`](../param/032_next.md), [`count::`](../param/037_count.md), [`offset::`](../param/038_offset.md), [`only_active::`](../param/039_only_active.md), [`only_next::`](../param/040_only_next.md), [`min_5h::`](../param/041_min_5h.md), [`min_7d::`](../param/042_min_7d.md), [`only_valid::`](../param/043_only_valid.md), [`exclude_exhausted::`](../param/044_exclude_exhausted.md), [`get::`](../param/045_get.md), [`abs::`](../param/046_abs.md), [`no_color::`](../param/047_no_color.md), [`live::`](../param/020_live.md), [`interval::`](../param/021_interval.md), [`jitter::`](../param/022_jitter.md), [`format::`](../param/002_format.md), [`trace::`](../param/023_trace.md)
-- **Exit:** 0 (success) | 1 (usage: invalid `name::` chars, legacy field-toggle param used, unknown `cols::` id, `for::` missing `@`, G8 ownership violation on `unclaim::1`) | 2 (runtime: account not found or credential store unreadable)

**Syntax:**

```bash
clp .accounts
clp .accounts name::alice@acme.com
clp .accounts alice@acme.com                         # positional: bare name at any position
clp .accounts car                                     # prefix: first saved account starting with "car"
clp .accounts cols::+host,-tier                      # add host column, remove tier column
clp .accounts cols::-owner                            # hide owner column
clp .accounts unclaim::1 name::alice@acme.com        # clear ownership (G8 gate)
clp .accounts unclaim::1 name::alice@acme.com force::1  # bypass G8
clp .accounts assign::1 name::alice@acme.com         # write active-account marker
clp .accounts assign::1                               # emit live usage block (no name::)
clp .accounts refresh::1                              # fetch live quota (HTTP)
clp .accounts refresh::1 sort::renew                 # sorted by renewal, live data
clp .accounts format::json
clp .accounts format::table
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/001_account_name.md) | *(omit to list all)* | Show or operate on a single named account; prefix resolution supported |
| `cols::` | `string` | `""` | Column visibility modifiers: comma-separated `+col_id` / `-col_id` relative to identity default set (`account`, `owner`, `active`, `current`, `sub`, `tier`, `expires`, `email`); opt-in: `display_name`, `host`, `role`, `billing`, `model`, `uuid`, `capabilities`, `org_uuid`, `org_name` |
| `unclaim::` | `bool` | `0` | Release ownership of `name::` account; writes `owner: ""` to `{name}.json`; G8 gate runs before write even when `dry::1`; when `name::` absent, batch-unclaims filtered set |
| `assign::` | `bool` | `0` | Write per-machine active-account marker for `name::` account; when `name::` absent, emits live usage block |
| `force::` | `bool` | `0` | Bypass G8 ownership gate on `unclaim::1`; allows any identity to release ownership; ignored without `unclaim::1` |
| `for::` | `string` (`USER@MACHINE`) | current `$USER@hostname` | Target machine identity for `assign::1`; split on first `@`; both components required when provided |
| `dry::` | `bool` | `0` | Preview mutations without writing; G8 gate still runs on `unclaim::1 dry::1` |
| `set_model::` | `enum` | *(omit)* | Write session model to `settings.json`: `opus`, `sonnet`, `haiku`, `default` |
| `refresh::` | `bool` | **`0`** | Attempt OAuth token refresh via subprocess (default `0`; differs from `.usage` default of `1`) |
| `touch::` | `bool` | **`0`** | Activate idle 5h session windows via subprocess (default `0`; differs from `.usage` default of `1`) |
| `imodel::` | `enum` | `auto` | Subprocess model: `auto`, `sonnet`, `opus`, `haiku`, `keep` |
| `effort::` | `enum` | `auto` | Subprocess effort: `auto`, `low`, `normal`, `high`, `max` |
| `sort::` | `enum` | **`name`** | Row ordering: `name` (default for `.accounts`), `renew`, `endurance`, `drain`, `next` |
| `desc::` | `bool` | `0` | Sort direction: 0 = ascending, 1 = descending |
| `prefer::` | `enum` | `any` | Weekly quota preference for sort strategies: `any`, `opus`, `sonnet` |
| `next::` | `enum` | `renew` | Recommendation strategy: `renew`, `endurance`, `drain` |
| `count::` | `u64` | `0` | Max rows to display (0 = all) |
| `offset::` | `u64` | `0` | Skip first N rows |
| `only_active::` | `bool` | `0` | Show only the per-machine active account |
| `only_next::` | `bool` | `0` | Show only the recommended next account |
| `min_5h::` | `f64` | `0` | Hide accounts with `5h Left` below this percentage |
| `min_7d::` | `f64` | `0` | Hide accounts with `7d Left` below this percentage |
| `only_valid::` | `bool` | `0` | Hide 🔴 (invalid/expired) rows |
| `exclude_exhausted::` | `bool` | `0` | Hide 🟡 and 🔴 rows |
| `get::` | `string` | `""` | Extract bare field value for first row |
| `abs::` | `bool` | `0` | Show absolute token counts instead of percentages |
| `no_color::` | `bool` | `0` | Strip emoji and ANSI colors |
| `live::` | `bool` | `0` | Continuous monitor mode |
| `interval::` | `u64` | `30` | Seconds between live refresh cycles (≥ 30) |
| `jitter::` | `u64` | `0` | Max random seconds added to interval |
| `format::` | [`OutputFormat`](../type/002_output_format.md) | `text` | Output format: `text`, `json`, `table` |
| `trace::` | `bool` | `0` | Print `[trace]` lines to stderr |

**Algorithm (5 steps):**
1. Resolve credential store; graceful degradation on unavailability (returns `(no accounts configured)` with exit 0)
2. List all accounts; resolve and filter by `name::` when provided
3. **Mutation dispatch:** `assign::1` → write per-machine marker or emit usage block (when `name::` absent); `unclaim::1` → evaluate G8 gate then write `owner: ""` or batch-unclaim filtered set; legacy field-toggle param present → exit 1 with `cols::` migration hint
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

clp .accounts format::table
# Accounts
#
#    Account         Owner        Active  Sub   Tier                     Expires
# -  --------------  -----------  ------  ----  -----------------------  ---------
# ✓  alice@acme.com  user1@w003   yes     max   default_claude_max_20x   in 2h 11m

clp .accounts unclaim::1 name::alice@acme.com
# unclaimed alice@acme.com

clp .accounts assign::1 name::alice@acme.com for::bob@laptop
# Assigned alice@acme.com for bob@laptop  →  _active_laptop_bob
```

**Notes:**
- `cols::` replaces the 15 former field-toggle params (`active::`, `current::`, `sub::`, `tier::`, `expires::`, `email::`, `display_name::`, `host::`, `role::`, `billing::`, `model::`, `uuid::`, `capabilities::`, `org_uuid::`, `org_name::`). Using any exits 1 with a `cols::` migration hint.
- Owner column is in the identity default set — shows `USER@MACHINE` when owned, `—` when unowned. Hide with `cols::-owner`.
- `format::json` always includes all fields regardless of `cols::`.
- `format::table` columns: flag, Account, Owner (when enabled), Active, Sub, Tier, Expires.
- Data-source params (`refresh::`, `touch::`) default to `0` — `.accounts` is local-only by default; set to `1` to activate the same live pipeline as `.usage`.
- `assign::` and `unclaim::` are also available on `.usage` (same behavior). See [Feature 037](../../feature/037_accounts_usage_param_unification.md).
- G8 ownership gate evaluates BEFORE `dry::1` on `unclaim::1` — a non-owner gets exit 1 even in dry-run mode.
- `current::` field (in text mode) shows `Current: yes` for the account whose `accessToken` matches `~/.claude/.credentials.json`. See [feature/016_current_account_awareness.md](../../feature/016_current_account_awareness.md).

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
| 8 | [Account Ownership](../../feature/036_account_ownership.md) | G8 gate for `unclaim::1`; `force::` bypass |
| 9 | [Accounts/Usage Param Unification](../../feature/037_accounts_usage_param_unification.md) | 32-param unified interface; `cols::` replacing field toggles; mutation params |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Rotation](../user_story/001_account_rotation.md) | Inspect accounts before and after rotation |
| 2 | [Account Onboarding](../user_story/002_onboarding.md) | Verify saved account metadata during onboarding |

---

### Command :: 4. `.account.save`

Copies `~/.claude/.credentials.json` to `{credential_store}/{name}.credentials.json` and merges identity, model, roles, and profile metadata into the unified `{name}.json`. Machine-global state (`commands.*`, `mcpServers`, `projects`) is not captured. Use this to preserve account identity before switching.

-- **Parameters:** [`name::`](../param/001_name.md), [`dry::`](../param/004_dry.md), [`host::`](../param/048_host.md), [`role::`](../param/052_role.md), [`trace::`](../param/023_trace.md)
-- **Exit:** 0 (success) | 1 (usage: invalid name or no active account set) | 2 (runtime: credentials unreadable)

**Syntax:**

```bash
clp .account.save                                      # infer name from oauthAccount.emailAddress in ~/.claude.json (falls back to _active_{hostname}_{user})
clp .account.save name::alice@acme.com                # explicit name
clp .account.save name::alice@acme.com dry::1
clp .account.save host::workstation                   # store host label in {name}.json
clp .account.save role::work                          # store role label in {name}.json
clp .account.save host::workstation role::personal    # both metadata fields
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/001_account_name.md) | `auto` (inferred from `oauthAccount.emailAddress` in `~/.claude.json`; falls back to per-machine active marker — see [Feature 025](../../feature/025_per_machine_active_marker.md); exits 1 if neither source present) | Account email to save as |
| `dry::` | `bool` | `0` | Preview action without executing |
| `host::` | `string` | `""` (auto-detected hostname) | Machine/host label stored in `{name}.json` (see [feature/029](../../feature/029_account_host_metadata.md)) |
| `role::` | `string` | `""` | User-defined role label stored in `{name}.json` (see [param 052](../param/052_role.md)) |
| `trace::` | `bool` | `0` | Print `[trace]` lines to stderr for credential read and file write steps |

**Algorithm (5 steps):**
1. Resolve `name::`: read `oauthAccount.emailAddress` from `~/.claude.json`; fall back to `_active_{hostname}_{user}` marker; exit 1 if neither present
2. `(when dry::0)` Copy `~/.claude/.credentials.json` → `{name}.credentials.json` (atomic write)
3. `(when dry::0)` Read `~/.claude.json` + `~/.claude/settings.json` + call `GET /api/oauth/claude_cli/roles` (best-effort); merge all into unified `{name}.json` (preserves `_renewal_at` and other keys)
4. `(when dry::0)` Write host, role, and owner into `{name}.json`: `host::` (auto-captured `$USER@$HOSTNAME` when omitted); `role::` via read-merge; `owner` = `current_identity()` always
5. `(when dry::0)` Write `_active_{hostname}_{user}` = `{name}` (per-machine active marker)

**Examples:**

```bash
clp .account.save
# saved current credentials as 'alice@acme.com'   (inferred from oauthAccount.emailAddress)

clp .account.save name::alice@acme.com dry::1
# [dry-run] would save current credentials as 'alice@acme.com'

clp .account.save host::workstation role::work
# saved current credentials as 'alice@acme.com'   (host='workstation', role='work')
```

**Notes:**
- Also writes `{credential_store}/_active_{hostname}_{user}` = `{name}` on every successful save (per-machine active marker via `active_marker_filename()`).
- Also calls endpoint 005 (`GET /api/oauth/claude_cli/roles`) and merges result into `{name}.json` (best-effort: failure is silently skipped).
- **Metadata refresh:** Re-running `.account.save` for an existing name refreshes the unified `{name}.json` and re-fetches endpoint 005 — this is the canonical way to refresh cached org identity without re-login. `{name}.json` is updated via read-merge (not full overwrite): the `oauthAccount` key is replaced but all other keys (e.g., `_renewal_at` set by `.account.renewal`) are preserved.
- **Ownership stamp:** `.account.save` always writes `current_identity()` as `owner` in `{name}.json` on every interactive save. Background refresh callers pass `owner: None`. To release ownership, use `clp .account.unclaim name::EMAIL` — calls `write_owner(name, store, "")` directly without touching credentials. See [Feature 036](../../feature/036_account_ownership.md).

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Store Init](../../feature/001_store_init.md) | Credential store initialization before save |
| 2 | [Save Account](../../feature/002_account_save.md) | Core save algorithm and file layout |
| 3 | [Persistent Storage](../../feature/010_persistent_storage.md) | Unified `{name}.json` merge semantics |
| 4 | [Per-Machine Active Marker](../../feature/025_per_machine_active_marker.md) | `_active_{hostname}_{user}` marker written on save |
| 5 | [Host Metadata](../../feature/029_account_host_metadata.md) | `host::` and `role::` metadata stored in `{name}.json` |
| 6 | [Account Ownership](../../feature/036_account_ownership.md) | Ownership model — `.account.save` stamps `current_identity()`; `.account.unclaim` clears ownership; `.account.assign` is marker-only |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Saving credentials during initial account setup |

---

### Command :: 5. `.account.use`

Atomically overwrites `~/.claude/.credentials.json` with the named account's credentials (write-then-rename), updates the active marker (`_active_{hostname}_{user}`), and best-effort patches `~/.claude.json["oauthAccount"]` from the saved snapshot — preserving all machine-global keys untouched. When `touch::1` (default), fetches quota for the target account and spawns an isolated subprocess to activate its idle 5h session window if `five_hour.resets_at` is absent.

-- **Parameters:** [`name::`](../param/001_name.md) **(required)**, [`dry::`](../param/004_dry.md), [`touch::`](../param/034_touch.md), [`refresh::`](../param/019_refresh.md), [`imodel::`](../param/035_imodel.md), [`effort::`](../param/036_effort.md), [`trace::`](../param/023_trace.md), [`set_model::`](../param/054_set_model.md)
-- **Exit:** 0 (success) | 1 (usage: invalid name or invalid `imodel::`/`effort::`/`trace::`/`set_model::` value) | 2 (runtime: account not found or HOME unset) | 3 (account credentials expired — `touch::1` + fetch failed + `expiresAt` in the past, AND refresh failed or `refresh::0`)

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
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/001_account_name.md) | **(required)** | Account email to switch to |
| `dry::` | `bool` | `0` | Preview action without executing |
| `touch::` | `bool` | `1` | Activate idle 5h session window via subprocess after switch |
| `refresh::` | `bool` | `1` | Attempt OAuth token refresh when locally expired before refusing with exit 3 |
| `imodel::` | `enum` | `auto` | Model for post-switch subprocess: `auto` (haiku — sufficient for keep-alive), `sonnet`, `opus`, `haiku`, `keep` |
| `effort::` | `enum` | `auto` | Effort for post-switch subprocess: `auto` (`low` for any model; no flag for haiku/keep), `low`, `normal`, `high`, `max` |
| `trace::` | `bool` | `0` | Print `[trace] account.use` lines to stderr: credential read, quota fetch, idle check, model resolution, subprocess dispatch |
| `set_model::` | `enum` | *(omit)* | Explicitly write session model to `settings.json`: `opus` (`claude-opus-4-6`), `sonnet` (`claude-sonnet-4-6`), `haiku` (`claude-haiku-4-5-20251001`), `default` (removes override); takes precedence over automatic `apply_model_override()` |

**Algorithm (7 steps):**
1. Resolve `name::` via `AccountSelector`; load `{name}.credentials.json`
2. `(when dry::0)` Atomically overwrite `~/.claude/.credentials.json` via write-then-rename
3. `(when dry::0)` Write `_active_{hostname}_{user}` = `{name}` (active marker)
4. `(when dry::0)` Best-effort patch `~/.claude.json["oauthAccount"]` from saved snapshot (preserves machine-global keys)
5. `(when touch::1)` Fetch quota via `GET /api/oauth/usage`; `(when refresh::1 + locally expired)` call `refresh_account_token()` first; evaluate idle: `five_hour.resets_at` absent → idle
6. `(when touch::1 + idle)` Resolve model+effort via `resolve_model()`/`resolve_effort()`; spawn isolated subprocess via `run_isolated()`
7. Session-model override: `(when set_model:: provided)` write requested model via `set_session_model()`; `(otherwise, when target was already active + valid quota)` write resolved model via `apply_model_override()`

**Examples:**

```bash
clp .account.use name::alice@home.com
# switched to 'alice@home.com'   (idle account: subprocess spawned to activate 5h session)

clp .account.use name::alice@home.com touch::0
# switched to 'alice@home.com'   (pure credential rotation — no subprocess)

clp .account.use name::alice@home.com dry::1
# [dry-run] would switch to 'alice@home.com'

clp .account.use name::alice@home.com trace::1
# [trace] account.use  alice@home.com  reading /...alice@home.com.credentials.json
# [trace] account.use  alice@home.com  reading: OK
# [trace] account.use  alice@home.com  quota fetch: OK
# [trace] account.use  alice@home.com  subprocess: scheduled (idle check removed)
# [trace] account.use  alice@home.com  model: claude-opus-4-6  effort: low
# [trace] account.use  alice@home.com  subprocess: spawned
# switched to 'alice@home.com'
```

**Notes:**
- `touch::1` (default): fetches quota for the target account; when fetch succeeds, always spawns `run_isolated(["--print", "."])` with resolved model/effort (subprocess is idempotent; Fix(BUG-285): idle check removed). Quota fetch failure checks `expiresAt` — if locally expired and `refresh::1` (default), attempts token refresh and re-probes touch context on success; exits 3 if refresh fails. If locally expired and `refresh::0`, exits 3 immediately. If `expiresAt` is absent or not yet expired, skips touch silently and the switch completes.
- `touch::0`: pure credential rotation — no quota fetch, no subprocess, no expiry check. Pre-Feature-027 behavior.
- `imodel::` and `effort::` follow the same resolution logic as `.usage` (Feature 026): `imodel::auto` always selects Haiku (sufficient for keep-alive pings); `resolve_effort()` maps Haiku and keep → no `--effort` flag, other models → `low`. See [feature/026](../../feature/026_subprocess_model_effort.md).
- `set_model::`: when provided, `set_session_model()` writes the requested model to `settings.json` last (after any `apply_post_switch_touch()` or `apply_model_override()`), ensuring it takes precedence. `default` removes the `model` key entirely.
- `trace::1` only produces output when `touch::1`; with `touch::0` there are no fetch operations to trace.
- See [feature/027_account_use_post_switch_touch.md](../../feature/027_account_use_post_switch_touch.md) for full execution sequence and acceptance criteria.

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Switch Account](../../feature/004_account_use.md) | Atomic credential rotation and active marker update |
| 2 | [Token Refresh](../../feature/017_token_refresh.md) | Pre-switch refresh on locally-expired token |
| 3 | [Session Touch](../../feature/024_session_touch.md) | Idle 5h window activation after switch |
| 4 | [Subprocess Model/Effort](../../feature/026_subprocess_model_effort.md) | Model and effort selection for post-switch subprocess |
| 5 | [Post-Switch Touch](../../feature/027_account_use_post_switch_touch.md) | Full execution sequence with touch and model override |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Rotation](../user_story/001_account_rotation.md) | Primary command for switching to a named account |

---

### Command :: 6. `.account.delete`

Removes `{credential_store}/{name}.credentials.json` and `{name}.json` from the credential store, plus any legacy satellite files from pre-consolidation layout.

-- **Parameters:** [`name::`](../param/001_name.md) **(required)**, [`dry::`](../param/004_dry.md), [`trace::`](../param/023_trace.md)
-- **Exit:** 0 (success) | 1 (usage: invalid name) | 2 (runtime: account not found)

**Syntax:**

```bash
clp .account.delete name::alice@oldco.com
clp .account.delete alice@oldco.com          # positional: bare name at any position
clp .account.delete dry::1 alice@oldco.com   # reversed: arg order does not matter
clp .account.delete car                      # prefix
clp .account.delete name::alice@oldco.com dry::1
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/001_account_name.md) | **(required)** | Account email to delete |
| `dry::` | `bool` | `0` | Preview action without executing |
| `trace::` | `bool` | `0` | Print `[trace]` lines to stderr for each file removal step |

**Algorithm (4 steps):**
1. Resolve `name::` via `AccountSelector`; validate account exists in credential store
2. `(when dry::0)` Delete `{name}.credentials.json`
3. `(when dry::0)` Best-effort delete `{name}.json` + legacy files (`.claude.json`, `.settings.json`, `.roles.json`, `.profile.json`; skip missing)
4. `(when dry::0 + deleted account = active)` Delete `_active_{hostname}_{user}` marker

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

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Delete Account](../../feature/005_account_delete.md) | File removal sequence and legacy satellite cleanup |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Removing stale accounts during account management |

---

### Command :: 11. `.account.limits`

Show rate-limit utilization for the active or named account. Displays session (5h) usage, weekly all-model (7d) usage, and rate-limit status with percentage consumed and reset times.

-- **Parameters:** [`name::`](../param/001_name.md) *(optional)*, [`format::`](../param/002_format.md), [`trace::`](../param/023_trace.md)
-- **Exit:** 0 (success) | 1 (usage: invalid `name::` chars) | 2 (runtime: account not found, data unavailable, HOME unset)

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
| `trace::` | `bool` | `0` | Print `[trace]` lines to stderr for credential store read and API call |

**Algorithm (3 steps):**
1. Resolve `name::` (omit → active account from `_active_{hostname}_{user}` marker); load credentials
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
```

**Notes:**
- Data source: `anthropic-ratelimit-unified-*` response headers; transport: `claude_quota::fetch_rate_limits()`. See [feature/013_account_limits.md](../../feature/013_account_limits.md).

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Account Limits](../../feature/013_account_limits.md) | Rate-limit header parsing and utilization rendering |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Multi-Account Quota Monitoring](../user_story/003_quota_monitoring.md) | Per-account rate-limit utilization check |

---

### Command :: 12. `.account.relogin`

Force browser-based re-authentication for a named account whose `refreshToken` is expired or revoked. This is the recovery path when `refresh::1` silently fails (trace shows `run_isolated: OK credentials=None` — Claude starts but performs no OAuth refresh because the refresh token itself is dead).

-- **Parameters:** [`name::`](../param/001_name.md) *(optional, defaults to active)*, [`dry::`](../param/004_dry.md), [`trace::`](../param/023_trace.md)
-- **Exit:** 0 (success: credentials refreshed and saved) | 1 (usage: invalid name value) | 2 (runtime: name omitted and no active account; account not found; or Claude spawn failed) | 3 (timeout or login abandoned: claude exited without updating credentials)

**Syntax:**

```bash
clp .account.relogin                   # default: active account
clp .account.relogin name::carol@example.com
clp .account.relogin carol@example.com          # positional: bare name at any position
clp .account.relogin dry::1 carol@example.com   # reversed: arg order does not matter
clp .account.relogin car               # prefix
clp .account.relogin name::carol@example.com dry::1
clp .account.relogin dry::1            # dry-run for active account
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/001_account_name.md) | *(active account)* | Account to re-authenticate; omit to use the currently active account |
| `dry::` | `bool` | `0` | Preview the steps without executing |
| `trace::` | `bool` | `0` | Print `[trace]` lines to stderr for each step: store read, switch, spawn, credential change, save, restore |

**Algorithm (6 steps):**
1. Resolve `name::` via [`AccountSelector`](../type/004_account_selector.md) → validate account exists in credential store
2. Snapshot the current active account name (for restoration after login)
3. `switch_account(name)` — makes the named account active in `~/.claude/`
4. Spawn `claude` with inherited TTY (stdin/stdout/stderr connected — NOT isolated subprocess) — Claude detects empty or invalid credentials and opens the browser login page
5. Wait for `claude` to exit; if `~/.claude/.credentials.json` changed → `account::save(name)` propagates fresh credentials to credential store
6. `switch_account(original_active)` — restore the prior active account

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
- Requires a TTY — `clp .account.relogin` in a piped non-TTY context will fail at step 4 (Claude cannot open a browser or display the login prompt).
- The `claude` subprocess runs with the full inherited environment; no credential isolation (contrast with `refresh::1` which uses an isolated subprocess).
- If `claude` exits without updating `~/.claude/.credentials.json`, the command exits 3. The active account is still restored (step 6 runs regardless of outcome).
- Use this when `clp .usage refresh::1 trace::1` shows `run_isolated: OK credentials=None` for an account — that trace indicates a dead refresh token requiring full browser re-auth.

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Auto Rotate](../../feature/008_auto_rotate.md) | Relogin as recovery path for dead refresh tokens |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Re-authenticating an account with expired refresh token |

---

### Command :: 13. `.account.rotate`

Auto-rotate to the best inactive account: selects the saved account with the highest remaining token expiry and atomically switches to it. No account name required — selection is fully automatic via `account::auto_rotate()`.

-- **Parameters:** [`dry::`](../param/004_dry.md), [`trace::`](../param/023_trace.md)
-- **Exit:** 0 (success: switched) | 1 (usage: invalid param) | 2 (runtime: no inactive accounts available, or credential store unreadable)

**Syntax:**

```bash
clp .account.rotate
clp .account.rotate dry::1
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `dry::` | `bool` | `0` | Preview which account would be selected without switching |
| `trace::` | `bool` | `0` | Print `[trace]` lines to stderr for account selection and switch step |

**Algorithm (4 steps):**
1. Enumerate all saved accounts; read `_active_{hostname}_{user}` marker
2. Filter to inactive accounts (name ≠ active marker value); exit 2 if none
3. Select account with highest `expiresAt` value (`account::auto_rotate()`)
4. `(when dry::0)` Execute `.account.use` steps for selected account

**Examples:**

```bash
clp .account.rotate
# rotated to 'alice@home.com'

clp .account.rotate dry::1
# [dry-run] would rotate to 'alice@home.com' (expires in 4h 12m, best of 2 inactive)

# When no inactive accounts exist (single account or all accounts = active):
clp .account.rotate
# exit 2: no inactive accounts available
```

**Notes:**
- Selects the inactive account with the highest `expiresAt` value across all saved credential files.
- "Inactive" = any saved account whose name differs from the active marker (`_active_{hostname}_{user}`).
- Equivalent to `clp .account.use $(best_account)` but without requiring the caller to determine the best candidate.
- If only one account is configured, or all saved accounts match the active one, exits 2.
- For explicit selection by name, use [`.account.use`](#command--5-accountuse).

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Auto Rotate](../../feature/008_auto_rotate.md) | Automatic best-account selection algorithm |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Rotation](../user_story/001_account_rotation.md) | Automatic rotation to the account with most remaining token life |

---

### Command :: 14. `.account.renewal`

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
| `trace::` | `bool` | `0` | Print `[trace]` lines to stderr for each file read and write step |

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

---

### Command :: 15. `.account.inspect`

Unified live account diagnostic — identity, subscription, org, and quota utilization for one account. Calls endpoints 002 (`GET /api/oauth/account`), 005 (`GET /api/oauth/claude_cli/roles`), and 001 (`GET /api/oauth/usage`) and renders identity fields (tagged_id, uuid, email, name), ALL membership entries with a selection-priority indicator, capabilities, rate-limit tier, and 5h/7d/Sonnet quota utilization with reset countdowns. Primary use case: diagnosing account state and remaining quota (see BUG-237 / feature 031).

-- **Parameters:** [`name::`](../param/001_name.md), [`refresh::`](../param/019_refresh.md), [`trace::`](../param/023_trace.md), [`format::`](../param/002_format.md)
-- **Exit:** 0 (success) | 1 (usage: invalid param) | 2 (runtime: account not found or credential store unreadable)

**Syntax:**

```bash
clp .account.inspect                    # default: active account
clp .account.inspect name::alice@acme.com
clp .account.inspect alice             # prefix
clp .account.inspect refresh::0        # skip token refresh on expired credentials
clp .account.inspect format::json
clp .account.inspect trace::1          # show [trace] endpoint calls to stderr
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/001_account_name.md) | *(active account)* | Account to inspect; omit to use the currently active account |
| `refresh::` | `bool` | `1` | Attempt OAuth token refresh via isolated subprocess when `expiresAt` is locally expired, before endpoint calls |
| `trace::` | `bool` | `0` | Print `[trace]` lines to stderr for each endpoint call: URL, HTTP status, field extraction summary |
| `format::` | [`OutputFormat`](../type/002_output_format.md) | `text` | Output format: `text` (default) or `json` |

**Algorithm (5 steps):**
1. Resolve `name::` (omit → active account from `_active_{hostname}_{user}` marker); load credentials
2. `(when refresh::1 + locally expired)` Call `refresh_account_token()` to obtain a fresh token
3. Call endpoint 002 (`GET /api/oauth/account`), endpoint 005 (`GET /api/oauth/claude_cli/roles`), and endpoint 001 (`GET /api/oauth/usage`) — each independently; failure falls back to local snapshots with `(snapshot)` suffix per field (quota fields show `N/A` with no snapshot fallback)
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
- Endpoints 002, 005, and 001 are called independently. A failure on one endpoint falls back to the local snapshot from `{name}.json` with a `(snapshot)` suffix per field; quota fields (endpoint 001) have no snapshot fallback and are omitted when unavailable.
- `refresh::1` (default) behaves identically to `.usage`'s `refresh::1`: calls `refresh_account_token()` once when `expiresAt` is locally expired; retries endpoint calls with the fresh token.
- See [feature/031_account_inspect.md](../../feature/031_account_inspect.md) for full design, graceful fallback semantics, and all acceptance criteria.

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Account Inspect](../../feature/031_account_inspect.md) | Unified account diagnostic — identity, subscription, org, and quota utilization |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | Live multi-endpoint inspection for subscription diagnosis |

---

### Command :: 16. `.account.assign` *(REMOVED — redirect stub)*

**Removed as of Feature 037.** This command exits 1 with a migration message. Use `.accounts assign::1 name::X` instead.

-- **Parameters:** [`name::`](../param/001_name.md) *(accepted but ignored)*, [`for::`](../param/053_for.md) *(accepted but ignored)*, [`dry::`](../param/004_dry.md) *(accepted but ignored)*, [`trace::`](../param/023_trace.md) *(accepted but ignored)*
-- **Exit:** 1 always — `"unknown command '.account.assign' — use '.accounts assign::1 name::X' instead"`

**Migration:**

```bash
# Before Feature 037:
clp .account.assign name::alice@corp.com
clp .account.assign name::alice@corp.com for::bob@laptop

# After Feature 037 (use .accounts assign::1):
clp .accounts assign::1 name::alice@corp.com
clp .accounts assign::1 name::alice@corp.com for::bob@laptop
clp .accounts assign::1                        # live usage block (no name::)
```

**Implementation note:** The command remains registered as a redirect stub (rather than deregistered) so callers receive this specific error message instead of a generic "unknown command" from the framework. The full original logic is in `account_assign_routine()` in `src/commands/account_assign.rs`.

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Account Assign](../../feature/032_account_assign.md) | Original marker-write algorithm — now absorbed as `.accounts assign::1` |
| 2 | [Accounts/Usage Param Unification](../../feature/037_accounts_usage_param_unification.md) | Feature that absorbed assign as mutation param and removed the standalone command |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Pre-configure active account for a remote machine |

---

### Command :: 17. `.account.unclaim` *(REMOVED — redirect stub)*

**Removed as of Feature 037.** This command exits 1 with a migration message. Use `.accounts unclaim::1 name::X` instead.

-- **Parameters:** [`name::`](../param/001_name.md) *(accepted but ignored)*, [`dry::`](../param/004_dry.md) *(accepted but ignored)*, [`trace::`](../param/023_trace.md) *(accepted but ignored)*
-- **Exit:** 1 always — `"unknown command '.account.unclaim' — use '.accounts unclaim::1 name::X' instead"`

**Migration:**

```bash
# Before Feature 037:
clp .account.unclaim name::alice@acme.com
clp .account.unclaim name::alice@acme.com dry::1

# After Feature 037 (use .accounts unclaim::1):
clp .accounts unclaim::1 name::alice@acme.com
clp .accounts unclaim::1 name::alice@acme.com dry::1
clp .accounts unclaim::1 name::alice@acme.com force::1  # bypass G8
clp .accounts unclaim::1                                 # batch-unclaim filtered set
```

**Implementation note:** The command remains registered as a redirect stub so callers receive this specific error message instead of a generic "unknown command". The unclaim logic lives in `accounts_routine()` in `src/commands/accounts.rs`.

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Account Ownership](../../feature/036_account_ownership.md) | Original unclaim algorithm (G8 gate, `write_owner()`) — now absorbed as `.accounts unclaim::1` |
| 2 | [Accounts/Usage Param Unification](../../feature/037_accounts_usage_param_unification.md) | Feature that absorbed unclaim as mutation param and removed the standalone command |

### Referenced User Stories

*None — ownership maintenance; no user-story-driven feature.*
