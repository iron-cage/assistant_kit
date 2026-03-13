# TSK-076 — Add typed builder methods for model and budget parameters

## Status

✅ (Completed)

## Metadata

- **Value:** 6
- **Easiness:** 8
- **Priority:** 2
- **Safety:** 8
- **Advisability:** 768

## Goal

Add typed `with_*` builder methods for `effort`, `fallback_model`, and `max_budget_usd`.

**MOST criteria:**
- **Motivated:** Cost control (`max_budget_usd`) and effort tuning are essential for production automation that must not overspend or underperform.
- **Observable:** Three new typed methods; `describe()` reflects correct flag syntax.
- **Scoped:** `src/command.rs` and `src/types.rs` only.
- **Testable:** `ctest3` green; tests verify correct flag rendering.

## Description

`--effort <level>` sets reasoning effort: `low`, `medium`, `high`, or `max`. `--fallback-model <model>` specifies an alternative model when the primary is overloaded. `--max-budget-usd <amount>` caps the API spend for the session.

An `EffortLevel` enum should be added to `src/types.rs` for type safety. `fallback_model` follows the same pattern as the existing `with_model()`. `max_budget_usd` takes a `f64`.

## In Scope

- `EffortLevel` enum: `Low`, `Medium`, `High`, `Max` — add to `src/types.rs`
- `with_effort(EffortLevel)` — adds `--effort <level>`
- `with_fallback_model<S: Into<String>>(model: S)` — adds `--fallback-model <model>`
- `with_max_budget_usd(amount: f64)` — adds `--max-budget-usd <amount>`
- Integration tests for all three methods
- Update `docs/claude_params/readme.md` Builder column for all three params

## Out of Scope

- Budget enforcement client-side (the flag is passed to claude; no local enforcement)

## Requirements

- Follow `code_design.rulebook.md` — TDD red-green-refactor
- Follow `codebase_hygiene.rulebook.md` — no mocking
- Follow `test_organization.rulebook.md` — tests in `tests/` only
- Follow `code_style.rulebook.md` — 2-space indent, custom codestyle

## Acceptance Criteria

- `with_effort(EffortLevel::High)` adds `--effort high`
- `with_effort(EffortLevel::Max)` adds `--effort max`
- `with_fallback_model("claude-haiku-4-5")` adds `--fallback-model claude-haiku-4-5`
- `with_max_budget_usd(0.50)` adds `--max-budget-usd 0.5`
- `ctest3` green; zero warnings

## Work Procedure

1. Read applicable rulebooks via `kbase .rulebooks`
2. Add `EffortLevel` enum to `src/types.rs`
3. Write failing tests for all three methods (RED)
4. Implement all three `with_*` methods in `src/command.rs`
5. All tests green (GREEN)
6. Refactor if needed; verify `ctest3`
7. Update `docs/claude_params/readme.md` Builder column
8. Update task status → ✅

## Validation Checklist

| # | Question | Desirable | Fail Action |
|---|----------|-----------|-------------|
| 1 | Does `with_effort(Low)` add `--effort low`? | YES | Fix enum `as_str()` |
| 2 | Does `with_fallback_model(s)` add `--fallback-model`? | YES | Fix method |
| 3 | Does `with_max_budget_usd(1.0)` add `--max-budget-usd 1`? | YES | Fix float formatting |
| 4 | Does `ctest3` pass? | YES | Fix failures |

## Validation Procedure

### Measurements

| # | Metric | Before | After | Command |
|---|--------|--------|-------|---------|
| M1 | Public `with_*` methods on `ClaudeCommand` | 41 | 44 | `grep -c 'pub fn with_' src/command.rs` |
| M2 | `ctest3` result | passing | passing | `ctest3` |

### Anti-faking Checks

- AF1: Build with `with_max_budget_usd(0.10)` and assert `describe()` contains `--max-budget-usd 0.1` — confirms float serialization is clean (no trailing zeros like `0.10000`)
