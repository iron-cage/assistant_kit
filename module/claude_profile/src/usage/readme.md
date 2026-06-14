# src/usage/

| File | Responsibility |
|------|----------------|
| `mod.rs` | Module declarations, public API re-exports, and shared test support. |
| `types.rs` | AccountQuota and strategy enums: SortStrategy, PreferStrategy, NextStrategy. |
| `fetch.rs` | Quota HTTP fetch, token reads, and per-account error formatting. |
| `format.rs` | Quota metric helpers: five_hour_left, prefer_weekly, renewal_secs. |
| `sort.rs` | Quota display sort strategies; sort_indices entry point. |
| `sort_next.rs` | Next-account recommendation strategies; find_next_for_strategy, strategy_metric. |
| `render.rs` | Text and JSON quota table rendering; render_text, render_json. |
| `live.rs` | Live loop mode: periodic refresh and re-render cycle. |
| `subprocess.rs` | Subprocess model and effort resolution; resolve_model, effort_pre_args. |
| `refresh.rs` | Token refresh loop for 401/403/expired accounts; apply_refresh. |
| `refresh_predicate.rs` | Refresh trigger predicate; should_refresh decision logic. |
| `touch.rs` | Session touch/probe: apply_touch, pre_switch_touch_ctx. |
| `params.rs` | Usage command parameter parsing and validation. |
| `api.rs` | Public command entry point: usage_routine. |
| `refresh_tests.rs` | Path-referenced tests for refresh.rs (apply_refresh). |
| `render_tests.rs` | Path-referenced tests for render.rs (render_text, render_tsv, render_json). |
| `sort_next_tests.rs` | Path-referenced tests for sort_next.rs (find_next_for_strategy, strategy_metric). |
| `touch_tests.rs` | Path-referenced tests for touch.rs (apply_touch). |
| `api_tests.rs` | Path-referenced tests for api.rs (pre_switch_touch_ctx, apply_model_override). |
| `format_tests.rs` | Path-referenced tests for format.rs (shorten_error, status_emoji, quota_text_cells). |

## Inline Test Exception

**Exception to `files_structure.rulebook.md § File Type Separation : Absolute Prohibitions`:**
All 12 source files in this module declare `#[cfg(test)] mod tests` — the test code
would ordinarily belong in `tests/`. This exception applies exclusively to `src/usage/`
and is justified by a visibility constraint.

**Two pattern variants in use:**
- **Inline blocks** (6 files): `#[cfg(test)] mod tests { ... }` — test code written
  directly in the source file.
- **Path-referenced files** (6 files: `format.rs`, `refresh.rs`, `render.rs`, `sort_next.rs`, `touch.rs`, `api.rs`):
  `#[cfg(test)] #[path = "*_tests.rs"] mod tests;` — test code lives in a companion
  `*_tests.rs` file in `src/usage/`. Both variants compile the tests as part of the
  crate module (not as external integration tests) so the visibility constraint is satisfied.

The functions under test (`pre_switch_touch_ctx`, `apply_model_override`, `apply_touch`,
`should_refresh`, `resolve_model`, `render_text`, `sort_indices`, etc.) are declared
`pub(crate)` — they are not part of the public API but must be tested in isolation.
Rust does not permit external test crates to access `pub(crate)` items across crate
boundaries, so moving the tests to `tests/` would require either:
- Widening visibility to `pub` (changes the public API surface), or
- Re-testing only through `pub` entry points (loses unit-level isolation)

Neither is acceptable here. The inline `#[cfg(test)]` blocks are the minimum viable
solution and are gated correctly — they produce no code in release builds.

**Scope:** This exception applies only to files within `src/usage/`. No other `src/`
directory in this crate is exempt from the Absolute Prohibition.
