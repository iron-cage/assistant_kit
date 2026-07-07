# Invariant: Shared Predicate Consistency

### Scope

- **Purpose**: Guarantee that any predicate combining multiple `AccountQuota`/`OauthAccountData` fields into one boolean conclusion is evaluated identically at every call site that relies on that conclusion.
- **Responsibility**: Documents the shared-predicate requirement, its concrete grounding in the `billing_type=="none"` + `result` combination (BUG-236/BUG-332), the detection method, and the enforcement/testing gap that allowed the two predicates to drift apart.
- **In Scope**: Any multi-field boolean/state predicate derived from `AccountQuota` (or its nested `account : Option<OauthAccountData>` / `result : Result<OauthUsageData, String>` fields) and consumed at ≥2 independent call sites across `src/usage/*.rs`.
- **Out of Scope**: Single-field checks with no combination logic (e.g., a bare `result.is_err()` with no accompanying `billing_type` check is not itself a "combined predicate" — it becomes one only when a second field is meant to gate the same conclusion); sort-key/tiebreak weighting (→ pitfall/001 Pitfall 1, unrelated axis); the specific display-string choice for cancelled accounts (→ algorithm/003, downstream of this invariant, not a redefinition of it).

### Invariant Statement

**Any boolean or state predicate that combines two or more underlying fields to answer a single logical question MUST be evaluated identically everywhere that question is asked** — either by calling one shared, named function, or by an explicit, testable guarantee (e.g., a data-layer override that structurally forces the fields into agreement before any downstream reader ever sees them) that keeps independently-written copies in sync. A literal condition re-typed at a second call site is not a guarantee — it is an assumption that has no mechanism forcing it to track the first site when the first site's logic changes.

**Concrete grounding — the `billing_type=="none"` + `result` predicate (BUG-236/BUG-332):**

The question "does this account have no active subscription?" is **not** answerable from `billing_type == "none"` alone. `billing_type == "none"` legitimately occurs for two distinct populations:

1. Genuinely cancelled subscriptions (the case `billing_type=="none"` was originally written to detect, BUG-232/233).
2. Non-stripe billing arrangements — team/enterprise accounts — where the usage API (`GET /api/oauth/usage`) still returns `Ok(...)` with valid quota data (BUG-236's finding).

The only signal that distinguishes population 1 from population 2 is the fetch **result** — `Err(...)` for genuinely dead accounts, `Ok(...)` for live non-stripe accounts. The correct, precise predicate is therefore the conjunction:

```
is_no_active_subscription(aq)  :=  aq.account.billing_type == "none"  AND  aq.result.is_err()
```

Evaluating `billing_type == "none"` in isolation — omitting the `result.is_err()` conjunct — answers a different, broader question ("is this a non-stripe-billed account?") while being silently substituted for the narrower one ("does this account have no active subscription?"). The two questions coincide for population 1 and diverge for population 2. Any call site that needs the narrower conclusion but evaluates only the broader condition will misclassify every population-2 account.

**Measurable threshold:** `grep -rn 'billing_type == "none"' src/usage/*.rs` MUST return zero call sites that (a) intend to answer "no active subscription" and (b) omit an accompanying `result.is_err()` (or equivalent shared-predicate call) in the same guard expression. As of BUG-332's filing, this crate has one call site where the conjunction is correctly enforced by construction (see Enforcement below) and multiple call sites where it is not (see Non-Compliant Sites).

### Enforcement Mechanism — Currently Structural, Not a Named Function

No function named `is_no_active_subscription()` (or equivalent) exists in this crate. The conjunction above is currently enforced by a **data-layer override**, not a shared predicate call:

**`src/usage/fetch.rs:255`** rewrites `AccountQuota.result` itself before it is ever stored:

```rust
let r = if account_data.as_ref().is_some_and( |a| a.billing_type == "none" ) && r.is_err() { Err( "no subscription".to_string() ) } else { r };
```

This is a **valid instance** of the invariant: it evaluates the full conjunction (`billing_type=="none" && r.is_err()`) at the one point where `result` is finalized, so any downstream reader that trusts `result` alone would see the conjunction's effect baked in. The guarantee this override provides is narrow and structural: **if** every downstream call site's actual need is fully satisfied by `result.is_err()` alone (post-override), no separate `billing_type` check is needed downstream at all.

**The invariant was violated (BUG-332) because that narrow guarantee was misapplied**: several downstream call sites re-derive `billing_type == "none"` independently, in parallel with `fetch.rs`'s override, instead of relying on the override's effect on `result`. Because these sites check `billing_type` directly rather than the already-corrected `result`, they see the *broader* condition (`billing_type=="none"`, true for both populations) instead of the *narrower* one the override guarantees (`result.is_err()`, true only for population 1 after the override runs). The override at `fetch.rs:255` and the re-derived checks below encode the same-looking literal condition to answer what look like the same question, but only the override's conjunctive form is precise — the re-derived single-field checks are not, and nothing forces them to stay aligned with the override's logic when it changes.

### Non-Compliant Sites (BUG-332's Concrete Evidence)

| Site | Code | Combines with `result`? | Consequence |
|------|------|:---:|-------------|
| `src/usage/render.rs:108` | `if aq.account.as_ref().is_some_and(\|a\| a.billing_type == "none") { "—" } else { renews_label(...) }` | No | Population-2 accounts (`billing_type=="none"`, `result=Ok`) show `"—"` ("no subscription") despite having a real, valid renewal date — BUG-332's reported symptom |
| `src/usage/render.rs:374` | Byte-identical duplicate of the above, inside `extract_get_field()`'s `GetField::Renews` arm | No | Same failure via the `get::renews` field-extraction path |
| `src/usage/render_tsv.rs:72` | Byte-for-byte duplicate of `render.rs:108`, same `Fix(BUG-232)` comment lineage | No | Same failure in the TSV/machine-readable renderer |
| `src/usage/sort.rs:47` | `status_group_of()` — gates `billing_type=="none"` to Dead group before quota thresholds | No (by design — see Related Cases) | Distinct symptom (status-group misclassification), not this bug's display cell, but the same broken single-field premise |
| `src/usage/sort_next.rs:36` | `find_first_eligible()` Gate 3b — skips cancelled accounts from next-account recommendation | No (by design — see Related Cases) | Distinct symptom (rotation exclusion), same broken premise |
| `src/usage/format.rs:273` | `sub_label()` — returns `"—"` for the `Sub` column | N/A — confirmed structurally distinct; `sub_label`'s signature never receives `aq.result` (see BUG-332 Dedup Search Record) | Not a violation of this invariant — `Sub` is intentionally result-agnostic by design |
| `src/usage/format.rs:491` | `status_emoji()` — gates `billing_type=="none"` to 🔴 before quota thresholds | No (by design — see Related Cases) | Distinct symptom (emoji color), same broken premise |
| `src/usage/api.rs:206` | `only_valid` filter — excludes `billing_type=="none"` accounts | Yes, but disjunctively (by design — see Related Cases) | Distinct symptom (row filtering), same broken premise |

**Related Cases — same broken single-field premise, deliberately out of this invariant's BUG-332 grounding:** `sort.rs:47`, `sort_next.rs:36`, `format.rs:491`, and `api.rs:206` all re-derive `billing_type == "none"` to answer a *different* question than "no active subscription" — namely "is this account permanently dead for classification/rotation/filtering purposes," a question BUG-317 (see pitfall/001 Pitfall 4) intentionally answers using `billing_type` alone as the dead-account signal, because a dead account should be excluded from rotation/coloring even if its last fetch happened to return `Ok`. `api.rs:206` additionally references `aq.result` in the same guard (`aq.result.is_ok() && !aq.account.as_ref().is_some_and(|a| a.billing_type == "none")`), but disjunctively — it excludes a row when *either* the fetch failed *or* `billing_type=="none"`, not the BUG-332 conjunction ("no active subscription" requires *both* to hold together). These sites are **not** BUG-332's failure mode and are not required to add a `result.is_err()` conjunct — they are listed here only because they share the same literal condition string, which is exactly the detection signal this invariant defines (see Detection below). Distinguishing "is this the BUG-332 predicate" from "is this a deliberately `result`-independent (or disjunctively `result`-combined) classification" requires checking what logical question the call site is actually answering, not just grep-matching the literal string.

### Detection

**Command:** `grep -rn 'billing_type == "none"' src/usage/*.rs`

**Interpretation procedure:**
1. For each match, identify the logical question the call site is answering.
2. If the question is "does this account have no active subscription" (i.e., the answer should be false for population 2 — live non-stripe accounts with `result=Ok`) — the match MUST also condition on `result.is_err()` (or read a field already corrected by `fetch.rs:255`'s override) in the same guard expression. Absence of this conjunct is a Non-Compliant Site per the table above.
3. If the question is a different, `result`-independent classification (dead-for-rotation, dead-for-coloring, dead-for-filtering — BUG-317's domain) — `billing_type` alone is the correct, complete predicate; no `result` conjunct is required or expected.
4. Any new call site added to this crate MUST be triaged through steps 2–3 before being merged; a re-derived literal condition that skips this triage is exactly the shape of drift BUG-332 documents.

**Sweep discipline:** when a fix narrows or refines the meaning of a combined predicate at one call site (as BUG-236 did at `fetch.rs:255`, adding the `result.is_err()` conjunct), every other call site sharing the same underlying field combination MUST be swept via the Detection command above and updated in the same change — not left for a later bug report to rediscover the drift.

### Violation Consequences

- A downstream renderer trusts `billing_type == "none"` alone as a proxy for "no active subscription," which was only ever a correct proxy for population 1 (genuinely cancelled accounts) — population 2 (live non-stripe accounts) is silently misclassified, producing a display value that actively asserts a wrong fact rather than an uninformative placeholder (BUG-332's `~Renews` shows `"—"` instead of a real renewal date).
- Per-file regression tests can each pass in isolation while the cross-file inconsistency between them goes undetected — `tests/usage/fetch_tests.rs`'s `mre_bug236_ok_result_not_overridden_when_billing_type_none` proves the fetch-layer conjunction holds, while `tests/usage/render_tests_a.rs`'s `test_ft23_009_renews_dash_for_cancelled_subscription` only ever constructs the `result=Err` case — neither test suite exercises the `billing_type=="none"` + `result=Ok` combination that the drift depends on.
- A grep for the literal condition string surfaces every candidate site, but distinguishing genuine violations (this invariant) from deliberately independent classifications (BUG-317's domain) requires the triage procedure in Detection step 2–3, not a mechanical string match alone.

### Sources

| File | Relationship |
|------|-------------|
| `src/usage/fetch.rs:248-255` | The one call site where the full conjunction (`billing_type=="none" && result.is_err()`) is correctly evaluated, via a data-layer override on `result` |
| `src/usage/render.rs:108,374` | Non-compliant — evaluates `billing_type` alone, ignoring `result` |
| `src/usage/render_tsv.rs:72` | Non-compliant — byte-for-byte duplicate of `render.rs:108`'s gate |
| `src/usage/types.rs:173-193` | `AccountQuota` struct — `account` and `result` are independent fields with no compiler-enforced link between them |
| `tests/usage/fetch_tests.rs:415-460` | `mre_bug236_ok_result_not_overridden_when_billing_type_none` — proves the fetch-layer conjunction is correct and the `billing_type=="none"` + `result=Ok` combination is reachable |
| `tests/usage/render_tests_a.rs:381` | `test_ft23_009_renews_dash_for_cancelled_subscription` — only tests the `result=Err` case; never tests the combination that exposes the drift |

### Bugs

| File | Relationship |
|------|--------------|
| BUG-236 | ✅ Fixed (TSK-242): narrowed the fetch-layer override at `fetch.rs:255` to require `billing_type=="none" && result.is_err()` — the correct instance of this invariant; never propagated to render-layer call sites |
| BUG-332 | ❓ Unverified (lifecycle state per task file header) — HIGH-confidence investigation pipeline finding with zero surviving counterexamples per the task file's own `## History`: `render.rs:108,374` and `render_tsv.rs:72` never inherited BUG-236's conjunction — this invariant's direct motivating case |
| BUG-317 | ✅ Fixed: added the same single-field `billing_type=="none"` check (deliberately `result`-independent) to `status_group_of()`, `find_first_eligible()`, `status_emoji()`, and `only_valid` — confirms the same literal condition has been independently re-derived in at least 4 more functions, none of which are violations of this invariant, but all of which are candidates the Detection command surfaces and the triage procedure must correctly classify |

### Algorithms

| File | Relationship |
|------|--------------|
| [algorithm/003](../algorithm/003_quota_status_groups.md) | Dead-group classification — a `result`-independent, correctly-scoped use of `billing_type=="none"`; also documents the BUG-332 pairing requirement for the `~Renews`-adjacent display logic |
| [algorithm/004](../algorithm/004_eligibility_gates.md) | Gate 3b — a `result`-independent, correctly-scoped use of `billing_type=="none"` |

### Pitfalls

| File | Relationship |
|------|--------------|
| [pitfall/001](../pitfall/001_quota_gate_pitfalls.md) | Pitfall 4 — documents the `result`-independent classification use of `billing_type=="none"` (BUG-317); this invariant documents the narrower, `result`-dependent "no active subscription" predicate that must NOT be evaluated the same way |
