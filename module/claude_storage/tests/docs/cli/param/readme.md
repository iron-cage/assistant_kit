# Parameter Tests

### Scope

- **Purpose**: Document edge case coverage for individual clg parameters.
- **Responsibility**: Index of per-parameter edge case test files covering parameter-level behavior.
- **In Scope**: All 34 clg parameter test files.
- **Out of Scope**: Command-level tests (→ `command/`), parameter group interactions (→ `param_group/`).

> **Known gap:** `26_depth.md` (the `depth::` parameter, documented at [`docs/cli/param/26_depth.md`](../../../../docs/cli/param/26_depth.md) and implemented for `.usage`) has no corresponding file in this directory — pre-existing staleness, not introduced by the `fields::`/`index::` additions (`32_fields.md`, `33_index.md`). Out of scope for this change; flagged here rather than silently left implicit.

### Responsibility Table

| Name | Purpose | Status |
|------|---------|--------|
| 01_agent.md | Edge case tests for `agent::` parameter | ✅ |
| 02_case_sensitive.md | Edge case tests for `case_sensitive::` parameter | ✅ |
| 03_entries.md | Edge case tests for `show_entries::` parameter | ✅ |
| 04_entry_type.md | Edge case tests for `entry_type::` parameter | ✅ |
| 05_format.md | Edge case tests for `format::` parameter | ✅ |
| 06_metadata.md | Edge case tests for `show_metadata::` parameter | ✅ |
| 07_min_entries.md | Edge case tests for `min_entries::` parameter | ✅ |
| 08_output.md | Edge case tests for `output::` parameter | ✅ |
| 09_path.md | Edge case tests for `path::` parameter | ✅ |
| 10_project.md | Edge case tests for `project::` parameter | ✅ |
| 11_query.md | Edge case tests for `query::` parameter | ✅ |
| 12_scope.md | Edge case tests for `scope::` parameter | ✅ |
| 13_session.md | Edge case tests for `session::` parameter | ✅ |
| 14_session_id.md | Edge case tests for `session_id::` parameter | ✅ |
| 15_sessions.md | Edge case tests for `show_sessions::` parameter | ✅ |
| 16_target.md | Edge case tests for `target::` parameter | ✅ |
| 17_topic.md | Edge case tests for `topic::` parameter | ✅ |
| 18_type.md | Edge case tests for `type::` parameter | ✅ |
| 19_show_stat.md | Edge case tests for `show_stat::` parameter | ✅ |
| 20_strategy.md | Edge case tests for `strategy::` parameter | ✅ |
| 21_count.md | Edge case tests for `count::` parameter | ✅ |
| 22_limit.md | Edge case tests for `limit::` parameter | ✅ |
| 23_show_tokens.md | Edge case tests for `show_tokens::` parameter | ✅ |
| 24_show_tree.md | Edge case tests for `show_tree::` parameter | ✅ |
| 25_last.md | Edge case tests for `last::` parameter | ✅ |
| 27_since_days.md | Edge case tests for `since_days::` parameter | ✅ |
| 28_show_topic.md | Edge case tests for `show_topic::` parameter | ✅ |
| 29_filter.md | Edge case tests for `filter::` parameter | ✅ |
| 30_detail.md | Edge case tests for `detail::` parameter | ✅ |
| 31_ids.md | Edge case tests for `ids::` parameter | ✅ |
| 32_fields.md | Edge case tests for `fields::` parameter | ✅ |
| 33_index.md | Edge case tests for `index::` parameter | ✅ |
| 42_full.md | Edge case tests for `full::` parameter | ✅ |
| 43_compact.md | Edge case tests for `compact::` parameter | ✅ |
