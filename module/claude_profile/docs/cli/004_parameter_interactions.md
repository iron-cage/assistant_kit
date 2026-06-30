# Parameter Interactions

Formal specification of co-dependencies, mutual exclusions, and cascading effects between clp parameters.

### All Interactions (12 total)

| # | Interaction | Parameters | Effect |
|---|-------------|------------|--------|
| 1 | `dry::` is orthogonal to output control | `dry::`, `format::` | Dry-run mode applies to mutation; does not affect output formatting |
| 2 | `format::json` overrides field-presence params | `format::`, `account::`, `sub::`, `tier::`, `token::`, `expires::`, `email::`, `file::`, `saved::`, `display_name::`, `role::`, `billing::`, `model::` | JSON output includes all fields regardless of field-presence param values |
| 3 | `format::table` ignores field-presence params | `format::`, `sub::`, `tier::`, `expires::`, `email::`, `display_name::`, `role::`, `billing::`, `model::` | Table output uses fixed columns regardless of field-presence param values |
| 4 | `live::1` is incompatible with `format::json` | `live::`, `format::` | Exits 1 before any fetch with `"live monitor mode is incompatible with format::json"` |
| 5 | `desc::` default is determined by `sort::` | `sort::`, `desc::` | Each sort strategy has a context-sensitive `desc::` default; explicit `desc::` overrides it |
| 6 | `prefer::` selects the weekly column used by sort heuristics | `sort::`, `prefer::` | `prefer::any/opus/sonnet` controls which weekly quota column `renew` strategy reads |
| 7 | `sort::` and `desc::` do not affect `format::json` output | `sort::`, `desc::`, `format::` | JSON array order is always alphabetical regardless of `sort::` or `desc::` (stable schema for pipeline consumers) |
| 8 | `cols::` does not affect `format::json` output | `cols::`, `format::` | JSON output is unaffected by column visibility modifiers; all schema fields always present |
| 9 | `sort::` recommendation does not affect `format::json` output | `sort::`, `format::` | JSON array order is always alphabetical; footer recommendation is omitted regardless of `sort::` value |
| 10 | `imodel::` and `effort::` do not affect `format::json` output | `imodel::`, `effort::`, `format::` | Subprocess model and effort control only subprocess invocations; JSON output structure is unchanged |
| 11 | `imodel::keep` + `effort::auto` injects no `--effort` flag | `imodel::`, `effort::` | When `imodel::keep`, no model is known at dispatch time; `effort::auto` resolves to no `--effort` flag to avoid incompatible model/effort combinations |
| 12 | `solo::1` is incompatible with `rotate::1` | `solo::`, `rotate::` | Exits 1 before any fetch — rotation requires live data from candidates but solo prevents live-fetching them |

---

### Interaction :: 1. `dry::` is orthogonal to output control

**Parameters:** [`dry::`](params.md#parameter--4-dry), [`format::`](params.md#parameter--2-format)

**Effect:** `dry::` controls whether the mutation executes; it does not affect output formatting. The `[dry-run]` prefix is always added to the confirmation message regardless of `format::`. The `dry::` parameter is only available on mutation commands (`.account.save`, `.account.use`, `.account.delete`), which do not belong to the Output Control group.

**Rationale:** Mutation commands produce single fixed-line confirmation messages, not parameterized output; the Output Control parameter has no effect on them. The two concerns — execution mode and output formatting — are fully independent.

**Commands affected:** [`.account.save`](commands.md#command--4-accountsave), [`.account.use`](commands.md#command--5-accountuse), [`.account.delete`](commands.md#command--6-accountdelete)

---

### Interaction :: 2. `format::json` overrides field-presence params

**Parameters:** [`format::`](params.md#parameter--2-format), [`account::`](params.md#parameter--5-account), [`sub::`](params.md#parameter--6-sub), [`tier::`](params.md#parameter--7-tier), [`token::`](params.md#parameter--8-token), [`expires::`](params.md#parameter--9-expires), [`email::`](params.md#parameter--10-email), [`file::`](params.md#parameter--11-file), [`saved::`](params.md#parameter--12-saved), [`display_name::`](params.md#parameter--14-display_name), [`role::`](params.md#parameter--15-role), [`billing::`](params.md#parameter--16-billing), [`model::`](params.md#parameter--17-model)

**Effect:** When `format::json` is specified on `.accounts` or `.credentials.status`, the JSON output always includes all fields regardless of field-presence param values. Setting `sub::0` only suppresses that field in text output, not in JSON.

**Rationale:** JSON consumers rely on stable schemas; selectively omitting fields based on presence params would break pipeline consumers that expect consistent structure. The field-presence params are a text-output formatting concern, not a data-selection concern.

**Commands affected:** [`.accounts`](commands.md#command--3-accounts), [`.credentials.status`](commands.md#command--10-credentialsstatus)

**Examples:**

```bash
# sub::0 suppresses Sub: in text, but JSON still has "subscription_type"
clp .accounts sub::0 format::json
# [{"name":"alice@acme.com","is_active":true,"subscription_type":"max",...}]  ← subscription_type still present

# file::0 (default) suppresses File: in text, but JSON always includes "file"
clp .credentials.status format::json
# {"subscription":"max",...,"file":"/home/user/.claude/.credentials.json","saved":2}
```

---

### Interaction :: 3. `format::table` ignores field-presence params

**Parameters:** [`format::`](params.md#parameter--2-format), [`sub::`](params.md#parameter--6-sub), [`tier::`](params.md#parameter--7-tier), [`expires::`](params.md#parameter--9-expires), [`email::`](params.md#parameter--10-email), [`display_name::`](params.md#parameter--14-display_name), [`role::`](params.md#parameter--15-role), [`billing::`](params.md#parameter--16-billing), [`model::`](params.md#parameter--17-model)

**Effect:** When `format::table` is specified on `.accounts`, the table always uses a fixed column set (flag, Account, Sub, Tier, Expires, Email) regardless of field-presence param values. Passing `sub::0` or `tier::0` alongside `format::table` has no effect on table columns.

**Rationale:** Table layout requires fixed column widths computed across all rows; selectively omitting columns based on field-presence params would break alignment and produce inconsistent table structures. Field-presence params are a text-output concern; table is a distinct, fixed-schema rendering mode.

**Commands affected:** [`.accounts`](commands.md#command--3-accounts)

**Examples:**

```bash
# sub::0 has no effect in table mode — Sub column still appears
clp .accounts sub::0 format::table

# All field-presence params ignored in table mode
clp .accounts sub::0 tier::0 format::table
```

---

### Interaction :: 4. `live::1` is incompatible with `format::json`

**Parameters:** [`live::`](params.md#parameter--20-live), [`format::`](params.md#parameter--2-format)

**Effect:** When both `live::1` and `format::json` are specified on `.usage`, the command exits 1 before any fetch with `"live monitor mode is incompatible with format::json"`.

**Rationale:** Live monitor mode requires interactive terminal control — ANSI screen clear (`\x1B[2J\x1B[H`) and a countdown footer line rewritten in-place via carriage return (`\r`). JSON output is a machine-readable, one-shot, newline-terminated format for pipeline consumption. Mixing the two rendering modes would corrupt JSON parsers with control codes and produce an unstable stream that no pipeline consumer could reliably parse. The guard runs once at startup before any network call.

**Commands affected:** [`.usage`](commands.md#command--9-usage)

**Examples:**

```bash
# Rejected before any fetch — exits 1
clp .usage live::1 format::json
# error: live monitor mode is incompatible with format::json

# Valid: live mode with default text format
clp .usage live::1 interval::60
# ...continuous monitor loop...

# Valid: single-shot JSON fetch without live mode
clp .usage format::json
# [...JSON array...]
```

---

### Interaction :: 5. `desc::` default is determined by `sort::`

**Parameters:** [`sort::`](param/025_sort.md), [`desc::`](param/026_desc.md)

**Effect:** When `sort::` is specified, the `desc::` default changes to match the strategy's natural direction. An explicit `desc::` always overrides the strategy default.

| `sort::` value | `desc::` default | Natural order |
|----------------|-----------------|---------------|
| `name` | `0` | A→Z |
| `renew` | `0` | Soonest reset on top |
| `renews` | `0` | Soonest billing renewal on top |

**Rationale:** Each strategy has a single natural direction that matches its workflow goal. Requiring explicit `desc::` in every invocation would be noisy; the default makes the common case require no extra flag.

**Commands affected:** [`.usage`](commands.md#command--9-usage)

**Examples:**

```bash
# sort::renew — desc::0 is the default (soonest reset on top)
clp .usage sort::renew
# same as: clp .usage sort::renew desc::0

# Override: reverse renew direction (latest reset on top)
clp .usage sort::renew desc::1
```

---

### Interaction :: 6. `prefer::` selects the weekly column used by sort tiebreak and recommendation heuristics

**Parameters:** [`sort::`](param/025_sort.md), [`prefer::`](param/027_prefer.md)

**Effect:** `prefer::` determines which weekly quota column is used by the `sort::renew` within-group tiebreak and the footer recommendation eligibility gate. `prefer::any` (default) uses `min(7d Left, 7d(Son))`; `prefer::opus` uses `7d Left`; `prefer::sonnet` uses `7d(Son)`.

**`prefer::` does NOT affect group membership.** The four-group status partition always uses raw `7d Left` for the weekly boundary (`7d Left > 5%` for Green/h-exhausted vs weekly-exhausted/Dead). An account's status group is determined by `5h Left` and `7d Left` columns only — not by `prefer_weekly`. See [AC-12](../../feature/020_usage_sort_strategies.md#acceptance-criteria).

**Affected heuristics:**
- `sort::renew` tiebreak: lowest `weekly(prefer)` ascending — within a group, among accounts with the same renewal event time, the account with the lower prefer-selected weekly quota ranks first
- Footer recommendation sort order: `prefer_weekly` determines tiebreak ranking, which indirectly determines which eligible account is recommended first. Eligibility itself uses model-agnostic `seven_day_left > WEEKLY_EXHAUSTION_THRESHOLD` (Fix BUG-324)

**Rationale:** Users who know they intend to run Opus or Sonnet can tell the sort tiebreak which model-specific quota to prefer. Group membership and eligibility are model-agnostic — it reflects raw quota availability, not a preference about which model to run.

**Commands affected:** [`.usage`](commands.md#command--9-usage)

**Examples:**

```bash
# Default: conservative weekly column for tiebreak
clp .usage sort::renew
# renew tiebreak uses min(7d Left, 7d(Son)) — group membership unaffected

# Opus sessions: only overall weekly quota matters for tiebreak
clp .usage sort::renew prefer::opus
# renew tiebreak uses 7d Left — group membership unaffected

# Sonnet sessions: Sonnet-specific weekly cap matters for tiebreak
clp .usage sort::renew prefer::sonnet
# renew tiebreak uses 7d(Son) — group membership unaffected
```

---

### Interaction :: 7. `sort::` and `desc::` do not affect `format::json` output

**Parameters:** [`sort::`](param/025_sort.md), [`desc::`](param/026_desc.md), [`format::`](param/002_format.md)

**Effect:** When `format::json` is specified, the JSON array order is unaffected by `sort::` or `desc::` — `render_json` preserves the input slice order without re-sorting. Alphabetical in practice because `fetch_all_quota` returns accounts alphabetically.

**Rationale:** JSON consumers rely on stable, predictable schemas. Row ordering is a visual/display concern for human-readable text output; injecting sort-strategy-dependent ordering into JSON would break pipeline consumers that expect consistent structure and make scripts fragile to `sort::` changes.

**Commands affected:** [`.usage`](commands.md#command--9-usage)

**Examples:**

```bash
# sort::renew has no effect on JSON array order
clp .usage sort::renew format::json
# [...array in fetch_all_quota order (alphabetical in practice)...]

# desc::1 has no effect on JSON array order
clp .usage sort::name desc::1 format::json
# [...array in fetch_all_quota order (alphabetical in practice)...]
```

---

### Interaction :: 8. `cols::` does not affect `format::json` output

**Parameters:** [`cols::`](param/033_cols.md), [`format::`](param/002_format.md)

**Effect:** When `format::json` is specified, column visibility modifiers from `cols::` have no effect. The JSON output always includes all schema fields regardless of which columns are shown in text-format table output.

**Rationale:** Column visibility is a text-format display concern — it controls which columns appear in the human-readable table. JSON consumers rely on a stable schema and must not receive partial objects based on display preferences. Injecting column-visibility decisions into JSON would break pipeline consumers that expect consistent structure.

**Commands affected:** [`.usage`](commands.md#command--9-usage)

**Examples:**

```bash
# cols::+sub has no effect on JSON schema
clp .usage cols::+sub format::json
# [...JSON array without "sub" key — JSON schema is fixed...]

# cols::-renews has no effect on JSON
clp .usage cols::-renews format::json
# [...JSON array with "renewal_secs", "renewal_is_estimate", "next_event_type", "next_event_secs" fields still present...]
```

---

### Interaction :: 9. `sort::` recommendation does not affect `format::json` output

**Parameters:** [`sort::`](param/025_sort.md), [`format::`](param/002_format.md)

**Effect:** When `format::json` is specified, the `sort::` recommendation has no effect. The JSON array order is always alphabetical and the footer recommendation is omitted — recommendation control is a text-format display concern.

**Rationale:** JSON consumers that parse `.usage` output need a stable, predictable array structure for scripting and automation. Injecting recommendation-dependent ordering into JSON would make scripts fragile to `sort::` changes. The footer recommendation is a human-readable text affordance; it has no JSON equivalent.

**Commands affected:** [`.usage`](commands.md#command--9-usage)

**Examples:**

```bash
# sort::renews has no effect on JSON array order
clp .usage sort::renews format::json
# [...array in fetch_all_quota order (alphabetical in practice)...]

# sort::renew (default) has no effect on JSON — no "strategy" fields injected
clp .usage format::json
# [...array without recommendation fields...]
```

---

### Interaction :: 10. `imodel::` and `effort::` do not affect `format::json` output

**Parameters:** [`imodel::`](param/035_imodel.md), [`effort::`](param/036_effort.md), [`format::`](param/002_format.md)

**Effect:** When `format::json` is specified, `imodel::` and `effort::` have no effect on JSON output structure. These parameters control which model and effort level are injected into isolated subprocesses spawned by `touch::` and `refresh::`; they do not alter the data rendered in the output.

**Rationale:** JSON consumers rely on a stable schema. Subprocess configuration (which model runs internally) is a fetch-behavior concern, not an output-structure concern. The JSON array fields are fixed regardless of how subprocesses are invoked.

**Commands affected:** [`.usage`](command/006_usage.md#command--9-usage)

**Examples:**

```bash
# imodel::opus has no effect on JSON structure
clp .usage imodel::opus format::json
# [...JSON array with standard fields — no model or effort fields...]

# effort::max has no effect on JSON output
clp .usage effort::max format::json
# [...JSON array identical to clp .usage format::json...]
```

---

### Interaction :: 11. `imodel::keep` + `effort::auto` injects no `--effort` flag

**Parameters:** [`imodel::`](param/035_imodel.md), [`effort::`](param/036_effort.md)

**Effect:** When `imodel::keep` is combined with `effort::auto`, no `--effort` flag is injected into subprocess args. The subprocess runs with neither `--model` nor `--effort` overrides.

**Rationale:** `effort::auto` resolves to `low` for any known model. When `imodel::keep`, the model is unknown at dispatch time; injecting an effort level without knowing the model risks unexpected behavior. The safe resolution is no effort override.

**Commands affected:** [`.usage`](command/006_usage.md#command--9-usage)

**Examples:**

```bash
# imodel::keep + effort::auto: no --model and no --effort injected
clp .usage imodel::keep effort::auto
# subprocess runs: claude --print .   (no model or effort overrides)

# imodel::keep + effort::high: --effort high is injected (explicit, model-independent)
clp .usage imodel::keep effort::high
# subprocess runs: claude --effort high --print .
```

---

### Interaction :: 12. `solo::1` is incompatible with `rotate::1`

**Parameters:** [`solo::`](param/060_solo.md), [`rotate::`](param/059_rotate.md)

**Effect:** When both `solo::1` and `rotate::1` are specified on `.usage`, the command exits 1 before any fetch with an error message referencing both `"solo"` and `"rotate"`. No table rendered.

**Rationale:** `rotate::1` needs live quota data from all candidate accounts to select the best switch target. `solo::1` prevents live-fetching any account except the current+owned one — candidates would have approximated data only, making rotation decisions unreliable. The two intents conflict: solo conserves tokens by avoiding API calls, while rotation requires API calls to make an informed decision.

**Commands affected:** [`.usage`](command/006_usage.md#command--9-usage)

**Examples:**

```bash
# Rejected before any fetch — exits 1
clp .usage solo::1 rotate::1
# error: solo::1 is incompatible with rotate::1

# Valid: solo without rotation
clp .usage solo::1
# ...table with live data for current+owned, approximated for others...

# Valid: rotation without solo
clp .usage rotate::1
# ...table + switch to recommended account...
```
