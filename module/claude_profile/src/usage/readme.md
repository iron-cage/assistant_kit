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
