# Commands: Identity

Identity commands: fleet-seat listing and per-Identity tag filter administration.

---

### Command: 23. `.identities`

Lists every [Identity](../../type/002_identity.md) (`user@machine`) observed anywhere in the credential store — the union of Identities appearing in `_active_*` markers, `_filter_*` files, and account `owner` fields — one sorted row per Identity with its active account, owned-account count, and tag filter halves. This is the fleet-seat overview the `user@host` concept never had.

-- **Parameters:** [`format::`](../param/002_format.md)
-- **Exit:** 0 (success, including empty result) | 1 (usage: unknown parameter, unsupported format) | 2 (runtime: credential store unreadable)

**Syntax:**

```bash
clp .identities
clp .identities format::json
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `format::` | [`OutputFormat`](../type/002_output_format.md) | `text` | Output format — `text` and `json` only |

**Columns:**

| Column | Content |
|--------|---------|
| Identity | `user@machine` |
| Active | Account named in the Identity's `_active_*` marker, or `—` |
| Owned | Count of accounts whose `owner` field equals the Identity |
| Include | Filter include set, comma-joined, or `—` (no filter file / empty side) |
| Exclude | Filter exclude set, comma-joined, or `—` |

**Algorithm (5 steps):**
1. Collect raw Identities: every distinct non-empty `owner` value across account profiles ([schema/002](../../schema/002_account_json.md)), plus the current Identity (`$USER@$HOSTNAME`, same resolution as [schema/005](../../schema/005_active_marker.md))
2. For each raw Identity, compute its sanitized `_active_{machine}_{user}` and `_filter_{machine}_{user}` filenames; claim any matching store files
3. For each unclaimed `_active_*`/`_filter_*` file, derive a display Identity from the filename suffix — split at the last `_` (user = final segment, machine = the rest), shown in sanitized form
4. For each Identity in the union: read active account name from its marker (or `—`), count owned accounts, read include/exclude from its filter file ([schema/009](../../schema/009_identity_filter_json.md), absent → both `—`)
5. Render one sorted row per Identity; when the union is empty print `(no identities)` and exit 0

**Examples:**

```bash
clp .identities
# Identity        Active           Owned  Include        Exclude
# alice@desk      alice@acme.com   2      ci,kimi_pool   —
# bob@laptop      —                1      —              personal

clp .identities format::json
# [{"identity":"alice@desk","active":"alice@acme.com","owned":2,"include":["ci","kimi_pool"],"exclude":[]},{"identity":"bob@laptop","active":null,"owned":1,"include":[],"exclude":["personal"]}]

clp .identities        # store with no markers, filters, or owners
# (no identities)
```

**Notes:**
- Read-only — never mutates markers, filters, or profiles.
- Rows derived only from a filename (step 3) show the sanitized halves — exact recovery is impossible when a raw half contained characters outside the marker charset; rows matched by construction (step 2) always show the raw `owner`-field form.
- An Identity appears even when only one source knows it — a filter file with no marker and no owned accounts still produces a row (that seat has declared a pool but never activated an account).

---

### Command: 24. `.identity.filter`

Get, set, or clear an Identity's [Tag Filter](../../type/004_tag_filter.md) — the per-seat include/exclude tag-set pair constraining automatic account selection (Gate 11, [algorithm/004](../../algorithm/004_eligibility_gates.md)). Without operation params, prints the target Identity's filter. With `include::`/`exclude::`, writes — each given side fully replaces that side. With `clear::1`, deletes the filter file (revert to permit-all). Filter files are store-resident (`_filter_{machine}_{user}`, [schema/009](../../schema/009_identity_filter_json.md)) and sync with the store; `identity::` administers another seat's filter centrally.

-- **Parameters:** [`include::`](../param/085_include.md), [`exclude::`](../param/086_exclude.md), [`clear::`](../param/051_clear.md), [`identity::`](../param/087_identity.md), [`format::`](../param/002_format.md)
-- **Exit:** 0 (success) | 1 (usage: `include ∩ exclude ≠ ∅`, invalid tag, `clear::1` with `include::`/`exclude::`, malformed `identity::`, unsupported format) | 2 (runtime: store unreadable/unwritable)

**Syntax:**

```bash
clp .identity.filter                                  # get (current Identity)
clp .identity.filter include::kimi_pool               # set include side
clp .identity.filter include::ci exclude::personal    # set both sides
clp .identity.filter clear::1                         # delete filter
clp .identity.filter identity::bob@laptop             # get another seat's filter
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `include::` | `string` | *(omit)* | Comma-separated tags an account must **all** carry; replaces the include side |
| `exclude::` | `string` | *(omit)* | Comma-separated tags an account must carry **none** of; replaces the exclude side |
| `clear::` | `bool` | `0` | Delete the Identity's filter file; idempotent; mutually exclusive with `include::`/`exclude::` |
| `identity::` | `string` | *(current Identity)* | Target Identity (`USER@MACHINE`) whose filter is read or written |
| `format::` | [`OutputFormat`](../type/002_output_format.md) | `text` | Output format (get mode only) — `text` and `json` only |

**Mode dispatch:**

| `include::`/`exclude::` | `clear::` | Mode |
|-------------------------|-----------|------|
| neither | `0` (default) | get — print the target Identity's filter; absent file → `include=[] exclude=[] (permit-all)` |
| either or both | `0` (default) | set — validate, replace the given side(s), write filter file |
| neither | `1` | clear — delete the filter file; success even when no file exists |
| either or both | `1` | error — exit 1; stderr: `clear:: is mutually exclusive with include::/exclude::` |

**Algorithm (get, 3 steps):**
1. Resolve target Identity — `identity::USER@MACHINE` if given (exit 1 unless exactly one `@` with both halves non-empty), else current `$USER@$HOSTNAME` per [schema/005](../../schema/005_active_marker.md) resolution
2. Read `_filter_{machine}_{user}`; absent file ≡ `{"include": [], "exclude": []}` (permit-all)
3. Render `include=[…] exclude=[…]` (suffix `(permit-all)` when both empty) in requested `format::` — JSON emits `{"identity": …, "include": […], "exclude": […]}`

**Algorithm (set, 6 steps):**
1. Resolve target Identity (as get step 1)
2. Normalize each given side per [type/003](../../type/003_tag.md): lowercase, validate charset `[a-z0-9_-]` and 1–64 length, deduplicate, sort — exit 1 naming the first offending tag, nothing written
3. Load the existing filter (absent ≡ both sides empty); replace each side that was given, keep the other
4. Reject contradiction: if `include ∩ exclude ≠ ∅`, exit 1 naming the overlapping tags — nothing written
5. Write the filter JSON per [schema/009](../../schema/009_identity_filter_json.md)
6. Typo guard: if the written include is non-empty and zero accounts satisfy `T ⊇ include`, print a stderr warning naming the tags carried by no account; exit stays 0

**Algorithm (clear, 2 steps):**
1. Resolve target Identity (as get step 1)
2. Delete `_filter_{machine}_{user}` if present; either way print confirmation and exit 0 (idempotent)

**Examples:**

```bash
clp .identity.filter
# include=[] exclude=[] (permit-all)

clp .identity.filter include::kimi_pool,ci
# include=[ci,kimi_pool] exclude=[]

clp .identity.filter exclude::personal
# include=[ci,kimi_pool] exclude=[personal]

clp .identity.filter format::json
# {"identity":"alice@desk","include":["ci","kimi_pool"],"exclude":["personal"]}

clp .identity.filter include::typo_tag
# include=[typo_tag] exclude=[personal]
# stderr: warning: no account carries tag(s): typo_tag — include can match nothing

clp .identity.filter include::a exclude::a
# exit 1: tag 'a' appears in both include and exclude

clp .identity.filter clear::1
# filter cleared (permit-all)

clp .identity.filter clear::1 include::ci
# exit 1: clear:: is mutually exclusive with include::/exclude::
```

**Notes:**
- The filter binds **automatic selection only** — `rotate::1`, auto-switch, and the footer `Next` recommendation (Gate 11, unconditional, no `force::1` interaction). `.account.use name::X` names its target and is never filtered ([feature/076](../../feature/076_identity_tag_filter.md) AC-09/AC-10).
- Unlike `_active_*` markers (machine-local), `_filter_*` files deliberately sync with the credential store — central administration via `identity::` depends on it ([schema/009](../../schema/009_identity_filter_json.md)).
- When Gate 11 excluded ≥1 account during a selection pass, `.usage` reports `N excluded by tag filter include=[…] exclude=[…]` — the filter announces itself where the surprise happens.
- Each write replaces only the side(s) given — setting `exclude::` never touches a previously written include, and vice versa.

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Identity Tag Filter](../../feature/076_identity_tag_filter.md) | Owning feature — filter semantics, Gate 11, both commands |
| 2 | [Account Tags](../../feature/075_account_tags.md) | Account-side tag sets the filter evaluates |

### Referenced Types

| # | Type | Role |
|---|------|------|
| 1 | [Identity](../../type/002_identity.md) | Row key of `.identities`; filter owner |
| 2 | [Tag Filter](../../type/004_tag_filter.md) | Value contract — structure, predicate, defaults |
| 3 | [Tag](../../type/003_tag.md) | Value type of both filter sets |

### Referenced Algorithms

| # | Algorithm | Role |
|---|-----------|------|
| 1 | [Eligibility Gates](../../algorithm/004_eligibility_gates.md) | Gate 11 — unconditional tag-mismatch exclusion the filter drives |

### Referenced Schema

| # | Schema | Role |
|---|--------|------|
| 1 | [Identity Filter JSON](../../schema/009_identity_filter_json.md) | `_filter_{machine}_{user}` file format both commands read/write |
| 2 | [Active Marker](../../schema/005_active_marker.md) | Sibling naming/sanitization convention; `.identities` marker source |
| 3 | [Account JSON](../../schema/002_account_json.md) | `owner` fields and `tags` arrays `.identities`/typo guard read |

### Referenced Formats

| # | Format | Trigger |
|---|--------|---------|
| 1 | [text](../format/001_text.md) | `format::text` (default) |
| 2 | [json](../format/002_json.md) | `format::json` |
