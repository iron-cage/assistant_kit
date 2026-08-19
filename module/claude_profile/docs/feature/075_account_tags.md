# Feature: Account Tags

### Scope

- **Purpose**: Let each saved account carry a *set* of normalized tags for fleet partitioning (e.g. `kimi_pool`, `ci`, `personal`) — so that operators can group accounts into overlapping operational pools, superseding the single free-form `role` label — and give tags first-class CLI surfaces for writing, listing, and filtering.
- **Responsibility**: Documents the `tags` array field in `{name}.json`; the `tags::` param on `.account.save`; the `.account.tag` mutation command (`add::`/`remove::`/`tags::`); the `.tags` listing command; the `.accounts` `tags::` subset filter, `Tags:` display line, and `cols::+tags` opt-in column; the `role::` save-param removal; and the lazy `role`→tag migration.
- **In Scope**: `tags: Vec<String>` field in `{name}.json`/`Account`/`AccountQuota` (absent ≡ empty set); tag normalization and rejection rules per [type/003](../type/003_tag.md) (charset `[a-z0-9_-]`, 1–64 chars, lowercase-then-validate, reject loudly); write paths `.account.save tags::` and `.account.tag add::/remove::/tags::` (ungated, comma-list `name::` batch, `dry::1` preview); `.tags` distinct-tag listing with account and filter-reference counts; `.accounts tags::` subset filter; `Tags:` text line and `cols::+tags` table column; `role::` save param REMOVED (exit 1 pointing at `tags::`); lazy per-account `role`→tag conversion on first tag write.
- **Out of Scope**: Filtering rotation eligibility by tags — that is [Feature 076](076_identity_tag_filter.md)'s per-Identity Tag Filter and Gate 11. Any tag registry, allow-list, or semantics — tags are free labels within the charset; meaning is assigned by whoever partitions the fleet. Any switch-time behavior change — env vars and credentials remain owned by `backend` ([Feature 071](071_redirect_backend_accounts.md)) and `inference_provider` ([Feature 072](072_inference_provider_selection.md)); tags never alter what `.account.use` writes.

### Design

**Why a set, not a scalar:** the legacy `role` field held exactly one free-form label, but operational pools overlap — one account can simultaneously be in `kimi_pool` and `ci`. A set (unordered, deduplicated, no intrinsic count limit) models this directly. Serialized order is sorted for deterministic files and clean diffs.

**Why normalization at write, rejection over coercion:** tag matching must be exact-string; silently normalizing (`Kimi Pool` → `kimi_pool`) would make the stored value differ from what the operator typed with no error, breaking later `tags::` filters typed the original way. So input is lowercased, then validated against `[a-z0-9_-]` (1–64 chars); anything still outside the charset exits 1 naming the offending tag. Full rules: [type/003](../type/003_tag.md).

**Why `role::` folds in:** two coexisting labeling mechanisms (free-form `role` + tags) are a term collision — every "which pool is this account in" question would need to check both. A role is just a tag with a cardinality restriction nobody needed. Migration is lazy and per-account: on the first tag write to an account whose `{name}.json` has a non-empty `role`, the role value is converted to a tag (lowercased, sanitized to the tag charset) and the `role` field is removed. No standalone migration pass; accounts never tag-written keep their `role` field indefinitely (still displayed by legacy `cols::+role`). The `role::` param on `.account.save` exits 1 with a migration message naming `tags::`.

**Why a dedicated `.account.tag` command:** tags change operationally far more often than accounts are re-saved — re-running `.account.save` re-captures live credentials as a side effect, which is wrong for a pure metadata edit. `.account.tag` provides add/remove/replace semantics against `{name}.json` only.

**Why ungated writes:** same doctrine as `lock::`/`reserve::` ([Feature 070](070_account_claim_and_reservation_control.md)) — tags are fleet-operations metadata, not credential operations; any machine may retag any account. `dry::1` previews; comma-list `name::X,Y,Z` batches.

**Mutation param semantics on `.account.tag`:**

| Params given | Effect |
|--------------|--------|
| `add::a,b` | Union into existing set (dedup, sort) |
| `remove::a` | Remove listed tags; removing an absent tag is a no-op success |
| `tags::a,b` | Replace the whole set |
| `add::` + `remove::` together | Exit 1 — one operation per invocation keeps outcomes predictable |
| `tags::` + (`add::` or `remove::`) | Exit 1 — replace is mutually exclusive with incremental ops |
| none of the three | Exit 1 — no operation given |

**Why `tags::` and not `set::` for replace:** param `set::` is RETIRED ([param 055](../cli/param/055_set.md), Feature 035); reviving it would resurrect a retired name with a second meaning. `tags::` already carries "the full tag set" semantics on `.account.save`, so reusing it as the replace form keeps one name for one concept.

**`.tags` listing:** enumerates every distinct tag across the fleet — the union of tags carried by accounts and tags referenced by Identity Tag Filters ([Feature 076](076_identity_tag_filter.md)) — with per-tag account count and filter-reference count. A tag referenced only by a filter shows `0` accounts, surfacing exactly the typo hazard Feature 076's write-time guard warns about.

**`.accounts` integration:** `tags::a,b` filters the listing to accounts whose tag set ⊇ `{a,b}` (subset semantics, matching the eligibility predicate's include half). Text mode renders a `Tags:` line only when the account carries ≥1 tag (no noise for untagged fleets); `format::json` always includes the `tags` array; table mode gets an opt-in `cols::+tags` column (comma-joined), available on both `.accounts` and `.usage` via the unified cols registry ([Feature 037](037_accounts_usage_param_unification.md)).

### Acceptance Criteria

- **AC-01**: `.account.save name::X tags::kimi_pool,ci` writes `"tags": ["ci", "kimi_pool"]` (deduplicated, sorted) into `X.json`. Exits 0.
- **AC-02**: `.account.save` without `tags::` leaves the `tags` field absent; every read path treats absence as the empty set.
- **AC-03**: A tag failing validation after lowercasing (charset `[a-z0-9_-]`, 1–64 chars, empty item in the comma list) exits 1 naming the offending tag; no file is written.
- **AC-04**: `.account.save role::x` exits 1 with a migration message naming `tags::`; no file is written.
- **AC-05**: `.account.tag name::X add::a,b` unions `{a, b}` into X's existing tag set (dedup, sorted). Exits 0.
- **AC-06**: `.account.tag name::X remove::a` removes `a`; removing a tag X does not carry is a no-op success (exit 0).
- **AC-07**: `.account.tag name::X tags::a,b` replaces X's whole tag set; combining `tags::` with `add::` or `remove::`, or combining `add::` with `remove::`, exits 1.
- **AC-08**: `.account.tag name::X` with none of `add::`/`remove::`/`tags::` exits 1 (no operation given).
- **AC-09**: The first tag write (via `.account.save tags::` or `.account.tag`) to an account whose `{name}.json` has a non-empty `role` converts that role value to a tag (lowercased, sanitized to the tag charset), merges it into the written set, and removes the `role` field from `{name}.json`.
- **AC-10**: `.account.tag` writes are ungated (no ownership check); `name::X,Y,Z` comma-list batches apply the same operation to each; `dry::1` previews all writes without touching disk.
- **AC-11**: `.tags` lists every distinct tag (union of account-carried and filter-referenced tags), sorted, with account count and filter-reference count per tag. Empty result prints `(no tags)` and exits 0.
- **AC-12**: `.tags format::json` emits a JSON array of `{"tag": ..., "accounts": N, "filters": N}` objects.
- **AC-13**: `.accounts tags::a,b` shows only accounts whose tag set contains **all** listed tags.
- **AC-14**: `.accounts` text mode shows a `Tags:` line (comma-joined, sorted) for accounts with ≥1 tag and omits it otherwise; `format::json` always includes the `tags` array.
- **AC-15**: `cols::+tags` adds a `Tags` column to `.accounts`/`.usage` table output; the column is not in any default set.
- **AC-16**: With no tags written anywhere, every pre-existing command behaves byte-identically to pre-feature behavior (zero-migration adoption).

### Bugs

_(none yet)_

### Domain Types

| File | Relationship |
|------|--------------|
| [type/003_tag.md](../type/003_tag.md) | Authoritative Tag value contract — charset, normalization, set semantics, `role` migration rules this feature implements |
| [type/001_account.md](../type/001_account.md) | Account — the aggregate carrying the tag set |

### Features

| File | Relationship |
|------|--------------|
| [076_identity_tag_filter.md](076_identity_tag_filter.md) | Companion feature — consumes tag sets via per-Identity include/exclude filters and Gate 11 |
| [029_account_host_metadata.md](029_account_host_metadata.md) | Origin of the `role` free-form label this feature supersedes; `host::` half of that feature is untouched |
| [070_account_claim_and_reservation_control.md](070_account_claim_and_reservation_control.md) | `lock::`/`reserve::` — structural precedent for ungated metadata writes with comma-list batch and `dry::1` |
| [072_inference_provider_selection.md](072_inference_provider_selection.md) | `inference_provider` — sibling save-time metadata label; orthogonal to tags (provider gates by Gate 10, tags by Gate 11) |
| [037_accounts_usage_param_unification.md](037_accounts_usage_param_unification.md) | Unified `cols::` registry the opt-in `tags` column joins |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/082_tags.md](../cli/param/082_tags.md) | `tags::` — write full set at save, replace set on `.account.tag`, subset filter on `.accounts` |
| [cli/param/083_add.md](../cli/param/083_add.md) | `add::` — union tags into the set on `.account.tag` |
| [cli/param/084_remove.md](../cli/param/084_remove.md) | `remove::` — remove tags from the set on `.account.tag` |
| [cli/param/052_role.md](../cli/param/052_role.md) | `role::` — REMOVED by this feature; exit-1 stub pointing at `tags::` |
| [cli/param/001_name.md](../cli/param/001_name.md) | `name::` — single target or comma-list batch on `.account.tag` |
| [cli/param/004_dry.md](../cli/param/004_dry.md) | `dry::` — preview `.account.tag` writes |

### Commands

| File | Relationship |
|------|--------------|
| [cli/command/001_account.md](../cli/command/001_account.md) | `.account.save` (`tags::` write path, `role::` removal), `.accounts` (`tags::` filter, `Tags:` line, `cols::+tags`), `.account.tag` (mutation command) |
| [cli/command/010_tag.md](../cli/command/010_tag.md) | `.tags` — distinct-tag listing command |

### Schema

| File | Relationship |
|------|--------------|
| [schema/002_account_json.md](../schema/002_account_json.md) | `tags` array field in `{name}.json`; `role` field deprecation and lazy removal |

### Sources

| File | Role |
|------|------|
| `../claude_profile_core/src/account/tags.rs` | Tag normalization/validation, set operations, `role`→tag lazy migration, `{name}.json` read/write (new) |
| `src/commands/account_tag.rs` | `.account.tag` and `.tags` command routines (new) |
| `src/commands/accounts.rs` | `tags::` filter, `Tags:` line, JSON field |
| `src/registry.rs` | Command and parameter registration |

### Tests

| File | Role |
|------|------|
| [tests/docs/feature/075_account_tags.md](../../tests/docs/feature/075_account_tags.md) | FT-level AC coverage plan |
| [tests/docs/cli/command/25_account_tag.md](../../tests/docs/cli/command/25_account_tag.md) | `.account.tag` integration test cases |
| [tests/docs/cli/command/22_tags.md](../../tests/docs/cli/command/22_tags.md) | `.tags` integration test cases |
| `tests/cli/account_tag_test.rs` | Integration test implementation |
