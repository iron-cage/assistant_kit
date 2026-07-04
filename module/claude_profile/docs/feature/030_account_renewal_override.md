# Feature: Account Billing Renewal Override

### Scope

- **Purpose**: Allow users to store an exact billing renewal timestamp for any account, correcting the approximate `org_created_at`-based estimate when the actual Stripe billing anchor differs.
- **Responsibility**: Documents the `.account.renewal` CLI command, `_renewal_at` field lifecycle in `{name}.json`, and its effect on `~Renews` column display in `.usage`.
- **In Scope**: `.account.renewal` command parameters (`name::`, `at::`, `from_now::`, `clear::`, `dry::`), `_renewal_at` ISO-8601 UTC field written to `{name}.json`, monthly auto-advance when `_renewal_at` is in the past, exact vs. estimated `~Renews` display, `save()` read-merge preserving `_renewal_at` across re-saves.
- **Out of Scope**: Stripe API integration (no subscription renewal endpoint is reachable via OAuth token); automatic detection of billing day from payment events; historical renewal tracking.

### Design

The Anthropic OAuth API exposes `org.created_at` as the billing cycle anchor, but this equals the organization creation date (typically free account creation), not the subscription start date. When users upgrade from free to paid after account creation, the Stripe billing anchor diverges from `org.created_at`, making the estimated renewal date wrong.

`.account.renewal` writes an optional `_renewal_at` ISO-8601 UTC string directly into `{name}.json` alongside the existing `oauthAccount` key. The `.usage` command reads this field during rendering and uses it as the authoritative renewal timestamp when present, falling back to the `org_created_at` estimate otherwise.

**`_renewal_at` field semantics:**
- Stored as a top-level key in `{name}.json`: `{"oauthAccount": {...}, "_renewal_at": "2026-06-29T21:00:00Z"}`
- When `_renewal_at` is in the past: auto-advanced by monthly increments until the result is in the future, using the same day-of-month from the original timestamp (Stripe-style billing cycle advance)
- When absent or `null`: falls back to `org_created_at`-based estimate (existing algorithm); `~Renews` shows `~` prefix
- `save()` preserves `_renewal_at` via read-merge (not overwrite) — see [002_account_save.md](002_account_save.md)

**`.account.renewal` operation steps:**
1. Resolve `name::`: `all` targets every account in the credential store; a comma-separated list targets the listed accounts; single email/prefix resolves via `AccountSelector`
2. Validate that exactly one of `at::`, `from_now::`, `clear::1` is provided (exits 1 on conflict or if none provided)
3. For each targeted account, read `{name}.json` (or start with empty object if absent)
4. Apply operation:
   - `at::VALUE`: parse ISO-8601, set `_renewal_at = VALUE`
   - `from_now::DELTA`: parse `±Xd Xh Xm` delta, compute `now ± delta` as ISO-8601 UTC, set `_renewal_at`
   - `clear::1`: remove `_renewal_at` key from the object
5. Write result to `{name}.json` (skips if `dry::1`)
6. Print one status line per account

**`at::` format:** ISO-8601 UTC datetime string. Accepts trailing `Z` or `+00:00` offset. Date-only form (`2026-06-29`) is interpreted as `2026-06-29T00:00:00Z`. Past values are accepted and auto-advanced on read.

**`from_now::` format:** Signed duration with `+`/`-` prefix. Supported units: `d` (days), `h` (hours), `m` (minutes). Examples: `+3h30m`, `-30m`, `+1d`, `+0m` (now). Multiple units may be combined (`+1d12h`). Computes `now ± delta` at command execution time and stores as ISO-8601 UTC.

**`clear::1`:** Removes `_renewal_at` from `{name}.json`. Restores `~`-prefixed estimate in `.usage`. Mutually exclusive with `at::` and `from_now::`.

**`dry::1`:** Prints the would-be operation and computed timestamp without writing any files.

**Multi-account:** `name::all` or comma list applies the same operation sequentially. Failures (account not found, parse error) are reported per-account and do not abort processing of remaining accounts.

**`~Renews` rendering in `.usage`:**

| Source | Condition | Display format | Example |
|--------|-----------|---------------|---------|
| `_renewal_at` | Present and auto-advanced to future | `in Xh Ym` (exact, no `~`) | `in 3h 47m` |
| `org_created_at` | `_renewal_at` absent, API data available | `~in Xd` (estimated, `~` prefix) | `~in 6d` |
| Neither | Both sources absent | `?` | `?` |

### Acceptance Criteria

- **AC-01**: `clp .account.renewal name::alice@acme.com at::2026-06-29T21:00:00Z` writes `_renewal_at: "2026-06-29T21:00:00Z"` into `{credential_store}/alice@acme.com.json`; the existing `oauthAccount` content is preserved.
- **AC-02**: `clp .account.renewal name::alice@acme.com from_now::+1h30m` writes `_renewal_at` as the current time plus 1 hour 30 minutes (ISO-8601 UTC); the existing `oauthAccount` content is preserved.
- **AC-03**: `clp .account.renewal name::alice@acme.com from_now::-30m` writes `_renewal_at` as the current time minus 30 minutes; `.usage` then auto-advances it monthly until future.
- **AC-04**: `clp .account.renewal name::alice@acme.com clear::1` removes `_renewal_at` from `{credential_store}/alice@acme.com.json`; `oauthAccount` content is preserved; `.usage` reverts to `~`-prefixed estimate.
- **AC-05**: `clp .account.renewal name::all from_now::+0m` writes the current timestamp into `_renewal_at` for every saved account; accounts without an existing `{name}.json` get a new file with only `{"_renewal_at": "..."}`.
- **AC-06**: `clp .account.renewal name::alice@acme.com at::2026-06-29T21:00:00Z dry::1` prints the would-be value without modifying any file.
- **AC-07**: `at::` and `from_now::` used together exits 1 with an error naming the conflict.
- **AC-08**: `at::` and `clear::` used together exits 1 with an error naming the conflict.
- **AC-09**: `from_now::` and `clear::` used together exits 1 with an error naming the conflict.
- **AC-10**: When `_renewal_at` is set to a past timestamp, `.usage` auto-advances it by monthly increments until the result is in the future, using the same day-of-month from the original `_renewal_at` timestamp.
- **AC-11**: When `_renewal_at` is present, `.usage` renders `~Renews` as `in Xh Ym` (exact duration, no `~` prefix); when absent and `org_created_at` is available, renders as `~in Xd` (with `~` prefix, 2 significant units max); when neither, renders `?`.
- **AC-12**: `clp .account.renewal name::alice@acme.com` with no `at::`, `from_now::`, or `clear::` provided exits 1 with a usage error.
- **AC-13**: `clp .account.renewal name::alice@acme.com at::2026-06-29T21:00:00Z` when `alice@acme.com` has no credential file exits 2 (account not found).
- **AC-14**: Comma-list `name::alice@acme.com,bob@acme.com` updates both accounts; output shows one status line per account.
- **AC-15**: Comma-list with one unknown account: the unknown account reports an error per-account; other accounts are still processed; command exits with a non-zero code reflecting the partial failure.

### Commands

| File | Relationship |
|------|--------------|
| [cli/command/001_account.md](../cli/command/001_account.md#command-14-accountrenewal) | CLI command specification |

### Features

| File | Relationship |
|------|--------------|
| [003_account_list.md](003_account_list.md) | `.accounts` — `renewal_at` field in design table and `format::json`; AC-21 |
| [002_account_save.md](002_account_save.md) | `save()` read-merge preserving `_renewal_at` (AC-17 there) |
| [009_token_usage.md](009_token_usage.md) | `.usage` rendering; `~Renews` and `→ Next` columns; AC-27/AC-28/AC-29 |
| [015_name_shortcut_syntax.md](015_name_shortcut_syntax.md) | Prefix resolution for `name::` — AC-12/AC-13 cover single and comma-list prefix resolution on `.account.renewal` |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/049_at.md](../cli/param/049_at.md) | `at::` — absolute renewal timestamp |
| [cli/param/050_from_now.md](../cli/param/050_from_now.md) | `from_now::` — relative renewal delta |
| [cli/param/051_clear.md](../cli/param/051_clear.md) | `clear::` — remove renewal override |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.renewal`](../cli/command/001_account.md#command-14-accountrenewal) | CLI surface for this feature |

### Sources

| File | Relationship |
|------|--------------|
| `src/account.rs` | `account_renewal()` — read-merge `_renewal_at` into `{name}.json`; multi-account dispatch |
| `src/commands/account_renewal.rs` | `account_renewal_routine()` — CLI handler; param validation; comma-list token resolution via `resolve_account_name()` |
| `src/usage/format.rs` | `renews_label()` — `~Renews` exact vs. estimated rendering; `next_event_label()` — `→ Next` event selection |

### Tests

| File | Relationship |
|------|--------------|
| `tests/cli/account_mutations_test.rs` | AC-01…AC-15 test cases |
