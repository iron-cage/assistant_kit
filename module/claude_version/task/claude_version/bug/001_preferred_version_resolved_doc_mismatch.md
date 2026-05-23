# BUG-001 — `preferredVersionResolved` Described as Authoritative Recovery Mechanism

## Symptom

Three primary documentation containers describe `preferredVersionResolved` as the recovery mechanism for version drift, while the actual implementation at `commands.rs:629-630` treats the stored value as advisory for alias specs — re-resolving through the live alias table instead. The advisory nature is documented only in a test pitfall comment, never in primary docs.

```
# docs/pattern/001_version_lock.md:40
"A recovery mechanism (preferredVersionResolved) is required to restore after
 accidental or automatic override"

# docs/feature/001_version_management.md:36
"preferredVersionResolved — concrete semver at install time, or null for latest"

# commands.rs:629-630 (actual behavior — alias re-resolution takes precedence)
let resolved_now = resolve_version_spec( spec );
let target = if resolved_now == spec { resolved } else { resolved_now };

# tests/integration/mutation_commands_test.rs:917-918 (ONLY location stating true semantics)
// Any code path that reads `preferredVersionResolved` for comparison
// must re-resolve the alias name first; the stored value is advisory.
```

## Impact

- **Who**: Any developer reading `docs/pattern/001_version_lock.md` or `docs/feature/001_version_management.md` to understand how `.version.guard` recovers from drift.
- **Conditions**: Whenever an alias (e.g. `month`, `stable`) is bumped after a version was stored. The stored `preferredVersionResolved` becomes stale; the guard silently uses the re-resolved value, not the stored one.
- **Severity**: Documentation-only defect — runtime behavior is correct (IT-15 fix is sound). Risk is incorrect mental model leading to incorrect future code paths that trust `preferredVersionResolved` as authoritative.

## How Discovered

Investigation of a `-problem.md` output audit for the `claude_version` module. Dual-agent validation (confirmatory + adversarial) on a Contradiction Signal: primary docs name `preferredVersionResolved` as the recovery mechanism while the guard implementation re-resolves the alias. Adversarial agent applied four strategies to disprove; all failed.

## MRE

1. Read `docs/pattern/001_version_lock.md:40` — states `preferredVersionResolved` is "A recovery mechanism."
2. Read `docs/feature/001_version_management.md:36` — states field is "concrete semver at install time."
3. Read `commands.rs:626-630` — `guard_once_pinned` receives `resolved` (the stored value) but immediately re-resolves via `resolve_version_spec(spec)`, using `resolved_now` if it differs from `spec`.
4. Write `settings.json` with `"preferredVersionResolved": "2.1.50"` and alias `month` currently pinned to `2.1.74` (TC-410 scenario).
5. Run `.version.guard dry::1` — output targets `v2.1.74`, not `v2.1.50`. Stored value ignored.
6. Contradiction: docs say stored value drives recovery; code says alias re-resolution drives recovery; stored value is advisory only.

## Root Cause

### Root Cause

The IT-15 alias re-resolution bug fix was applied correctly to `commands.rs:629-630` — `guard_once_pinned` now re-resolves the alias through the live compile-time table before comparing against the installed version. This makes `preferredVersionSpec` (the alias name) the true recovery driver: the guard re-resolves it to the current semver and uses that as the target. `preferredVersionResolved` is only used as a fallback when `spec` is already a concrete semver (i.e., `resolve_version_spec(spec) == spec`), making it advisory for alias specs.

However, the three primary documentation containers were not updated to reflect this semantic change:

1. `docs/pattern/001_version_lock.md:40` — Layer 5 Applicability section still names `preferredVersionResolved` as the recovery mechanism (not `preferredVersionSpec`).
2. `docs/pattern/001_version_lock.md:28` — Layer 5 Solution description does not mention alias re-resolution at guard time.
3. `docs/feature/001_version_management.md:36` — field description omits the advisory qualifier for alias specs.
4. `docs/feature/001_version_management.md:38-44` — Version guard steps omit the alias re-resolution step.

The true recovery path is: `preferredVersionSpec` (alias name) → re-resolved via `resolve_version_spec()` at guard time → used as `target`. `preferredVersionResolved` is advisory: authoritative only when `spec` is a concrete semver (the alias table lookup returns the spec unchanged).

Propagation: any developer following the primary docs to implement a new code path that reads `preferredVersionResolved` as the comparison baseline would introduce the pre-IT-15 bug.

### Why Not Caught

The IT-15 fix was applied in a code-first pass. The fix itself (TC-410) was documented in the test file with correct semantics, but the primary documentation containers (`docs/pattern/`, `docs/feature/`) were not part of the fix scope — no doc-consistency pass was performed after the code fix.

### Fix Location

1. `docs/pattern/001_version_lock.md:40` — change "A recovery mechanism (`preferredVersionResolved`) is required…" to name `preferredVersionSpec` as the recovery driver; note that `preferredVersionResolved` is advisory for alias specs.
2. `docs/pattern/001_version_lock.md:28` — Layer 5 Solution description: add "at guard time, the alias name in `preferredVersionSpec` is re-resolved through the current alias table to determine the target semver."
3. `docs/feature/001_version_management.md:36` — `preferredVersionResolved` field description: add "advisory for alias specs (guard re-resolves `preferredVersionSpec` through the current alias table); authoritative for concrete semver specs."
4. `docs/feature/001_version_management.md:38-44` — Version guard steps: add step 3a or inline note: "For alias specs, re-resolve `preferredVersionSpec` through the current alias table — `preferredVersionResolved` is not used as the comparison target."
5. `src/commands.rs:625-630` — `guard_once_pinned` doc comment: note that `resolved` is advisory for alias specs; the function re-resolves `spec` through `resolve_version_spec()` and uses `resolved_now` when it differs.

### Prevention

Whenever a bug fix changes field semantics (from "authoritative" to "advisory"), a documentation consistency pass is required over all primary doc containers (`docs/feature/`, `docs/pattern/`, `docs/invariant/`, source doc comments) before the fix is considered complete.

**Pitfall:** A field that is authoritative for one spec type (concrete semver) may be advisory for another (alias). If the distinction is documented only in a test pitfall comment and not in primary docs, future implementers will treat it as universally authoritative and re-introduce the pre-fix bug.

### Generalized Version

**Pattern: Code-first fix leaves primary docs describing pre-fix semantics.**

A bug fix correctly changes runtime behavior but does not update the primary documentation containers. The fix is documented in the test file (closest to the fix site) but the feature and pattern docs still describe the old semantics. Any implementer reading the primary docs — rather than the test file — builds a mental model of the pre-fix behavior. Future code written under that model re-introduces the bug. The canonical form: "the fix was local; the docs were global; only the local got updated."

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|------------|-------|---------|----------|
| H1 | `preferredVersionResolved` is authoritative — docs are correct, guard uses stored value | DISPROVED | `commands.rs:629-630` re-resolves alias before using stored value; TC-410 demonstrates stale stored value is overridden | `commands.rs:629-630`, `mutation_commands_test.rs:891-937` |
| H2 | Advisory nature is documented in primary docs somewhere not yet searched | DISPROVED | Adversarial agent ran 4 strategies including full-text search across `docs/`; only location is test pitfall comment at `mutation_commands_test.rs:917-918` | `mutation_commands_test.rs:917-918`; absence confirmed across `docs/pattern/`, `docs/feature/`, `src/` doc comments |
| H3 | The doc description is intentionally simplified (alias case is edge-case) | DISPROVED | IT-15 was a real bug; the alias re-resolution was the explicit fix; the distinction is not an edge case but the primary motivation for the fix | `mutation_commands_test.rs:891` — TC-410 header names this the fix |
| H4 | `guard_once_pinned` doc comment correctly describes advisory semantics | DISPROVED | `commands.rs:625` doc comment says "compare installed vs preferred and restore on drift" — no mention of re-resolution or advisory status | `commands.rs:625-626` |

## Evidence Table

| Location | What It Shows |
|----------|---------------|
| `docs/pattern/001_version_lock.md:40` | Names `preferredVersionResolved` as "A recovery mechanism" — contradicts implementation |
| `docs/pattern/001_version_lock.md:28` | Layer 5 describes storing both fields; no mention of alias re-resolution at guard time |
| `docs/feature/001_version_management.md:36` | Defines `preferredVersionResolved` as "concrete semver at install time, or null for latest" — no advisory qualifier |
| `docs/feature/001_version_management.md:38-44` | Version guard steps list 4 steps; no step for alias re-resolution |
| `commands.rs:626` | `guard_once_pinned` doc comment: "compare installed vs preferred and restore on drift" — no mention of advisory semantics |
| `commands.rs:628-630` | Re-resolution logic: `resolved_now = resolve_version_spec(spec)`; `target = if resolved_now == spec { resolved } else { resolved_now }` — `resolved` (stored value) used only as fallback |
| `mutation_commands_test.rs:891` | TC-410 header: "stale `preferredVersionResolved` → guard re-resolves alias" — correct semantics, but test-only |
| `mutation_commands_test.rs:917-918` | **Only** primary statement that stored value is advisory: "the stored value is advisory" |

## History

| Date | Event | Note |
|------|-------|------|
| 2026-05-23 | filed | Source: `/bugs -problem.md` output audit; dual-agent validation (confirmatory CONFIRMED, adversarial UNABLE_TO_DISPROVE) |
