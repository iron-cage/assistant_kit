# src/usage/

| File | Responsibility |
|------|----------------|
| `mod.rs` | Module declarations, public API re-exports, and shared test support. |
| `types.rs` | AccountQuota and strategy enums: SortStrategy, PreferStrategy. |
| `fetch.rs` | Quota HTTP fetch, token reads, and per-account error formatting. |
| `fetch_cache.rs` | Centralized quota cache read with polynomial approximation. |
| `approx.rs` | Quota measurement polynomial approximation (Feature 040). |
| `format.rs` | Quota metric helpers: five_hour_left, prefer_weekly, renewal_secs. |
| `sort.rs` | Quota display sort strategies; sort_indices entry point. |
| `sort_next.rs` | Next-account recommendation strategies; find_next_for_strategy, strategy_metric. |
| `render.rs` | Text and JSON quota table rendering; render_text, render_json. |
| `render_json.rs` | JSON renderer for quota results. |
| `render_sessions.rs` | Sessions marker table for quota output footer. |
| `render_tsv.rs` | TSV renderer for quota results. |
| `live.rs` | Live loop mode: periodic refresh and re-render cycle. |
| `subprocess.rs` | Subprocess model and effort resolution; resolve_model, effort_pre_args. |
| `refresh.rs` | Token refresh loop for 401/403/expired accounts; apply_refresh. |
| `refresh_predicate.rs` | Refresh trigger predicate; should_refresh decision logic. |
| `touch.rs` | Session touch/probe: apply_touch, pre_switch_touch_ctx. |
| `params.rs` | Usage command parameter parsing and validation. |
| `api.rs` | Public command entry point: usage_routine. |
| `api_dispatch.rs` | Mutation-dispatch helpers for usage command routines. |
| `api_switch.rs` | Pre/post-switch touch context and model-override helpers. |
