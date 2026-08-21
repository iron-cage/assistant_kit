# Parameters

### Scope

- **Purpose**: Document individual parameter specifications for the `claude_storage` CLI.
- **Responsibility**: Per-parameter detail pages with type, defaults, and command cross-refs.
- **In Scope**: All 40 CLI parameters with type constraints, defaults, valid values, and command usage.
- **Out of Scope**: Type definitions (→ `type/`), parameter group semantics (→ `param_group/`), command-level behavior (→ `command/`).

All parameters for the `claude_storage` CLI. Parameters use `param::value` syntax.

See [type/readme.md](../type/readme.md) for type definitions and [param_group/readme.md](../param_group/readme.md) for related parameter sets.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `01_agent.md` | agent:: — session type filter (main vs agent) |
| `02_case_sensitive.md` | case_sensitive:: — case-sensitive search toggle |
| `03_entries.md` | show_entries:: — show all session entries flag |
| `04_entry_type.md` | entry_type:: — filter search by entry type |
| `05_format.md` | format:: — export output format selector |
| `06_metadata.md` | show_metadata:: — show metadata only flag |
| `07_min_entries.md` | min_entries:: — minimum entry count threshold |
| `08_output.md` | output:: — export output file path |
| `09_path.md` | path:: — path argument (semantics vary by command) |
| `10_project.md` | project:: — project scope identifier |
| `11_query.md` | query:: — search query string (required by .search) |
| `12_scope.md` | scope:: — session/project discovery scope |
| `13_session.md` | session:: — session filter or scope pin |
| `14_session_id.md` | session_id:: — direct session identifier |
| `15_sessions.md` | show_sessions:: — explicit session display toggle |
| `16_target.md` | target:: — count operation target |
| `17_topic.md` | topic:: — session topic suffix |
| `18_type.md` | type:: — project naming scheme filter |
| `19_show_stat.md` | show_stat:: — session statistics footer toggle |
| `20_strategy.md` | strategy:: — resume strategy override |
| `21_count.md` | count:: — output count only flag (`.list`, deprecated; `.projects`) |
| `22_limit.md` | limit:: — per-project session display cap |
| `23_show_tokens.md` | show_tokens:: — token usage section toggle |
| `24_show_tree.md` | show_tree:: — agent tree-indented display toggle |
| `25_last.md` | last:: — trailing entry count for .tail and .show's project overview |
| `26_depth.md` | depth:: — path-component depth cap for .usage scope walks |
| `27_since_days.md` | since_days:: — recency window in days for .projects |
| `28_show_topic.md` | show_topic:: — first user message on session lines toggle |
| `29_filter.md` | filter:: — path-substring filter on resolved projects for .projects |
| `30_detail.md` | detail:: — output detail level (projects vs sessions) for .projects and .show |
| `31_ids.md` | ids:: — raw conversation-ID scripting output toggle for .projects |
| `32_fields.md` | fields:: — attribute-projection field selector for .show |
| `33_index.md` | index:: — single-message position selector for .show |
| `34_group.md` | group:: — aggregation dimension for .rollup |
| `35_sort.md` | sort:: — sort column for .rollup's grouped rows |
| `36_order.md` | order:: — sort direction for .rollup |
| `37_model.md` | model:: — model-name substring filter for .rollup |
| `38_columns.md` | columns:: — column projection for .rollup |
| `39_session_ids.md` | session_ids:: — cross-project conversation selector for .cost |
| `40_agents.md` | agents:: — agent fold-in toggle for .cost |

### Parameters Table

| # | Parameter | Type | Default | Valid Values | Purpose | Used In |
|---|-----------|------|---------|-------------|---------|---------|
| 1 | [`agent::`](01_agent.md) | Boolean | — | `0`, `1` | Session type filter (main vs agent) | 2 |
| 2 | [`case_sensitive::`](02_case_sensitive.md) | Boolean | `0` | `0`, `1` | Case-sensitive search matching | 1 |
| 3 | [`show_entries::`](03_entries.md) | Boolean | `0` | `0`, `1` | Show all session entries | 1 |
| 4 | [`entry_type::`](04_entry_type.md) | [`EntryType`](../type/02_entry_type.md) | `all` | `user`, `assistant`, `all` | Filter search by entry type | 1 |
| 5 | [`format::`](05_format.md) | [`ExportFormat`](../type/03_export_format.md) | `markdown` | `markdown`, `json`, `text` | Export output format | 1 |
| 6 | [`show_metadata::`](06_metadata.md) | Boolean | `0` | `0`, `1` | Show metadata only mode | 1 |
| 7 | [`min_entries::`](07_min_entries.md) | [`EntryCount`](../type/01_entry_count.md) | — | Integer ≥ 0 | Minimum entry count threshold | 2 |
| 8 | [`output::`](08_output.md) | [`StoragePath`](../type/10_storage_path.md) | — | filesystem path | Export output file path | 1 |
| 9 | [`path::`](09_path.md) | varies | varies | varies | Path argument (semantics vary by command) | 15 |
| 10 | [`project::`](10_project.md) | [`ProjectId`](../type/05_project_id.md) | current dir | path, uuid, substring | Project scope identifier | 5 |
| 11 | [`query::`](11_query.md) | String | — | any string | Search query string | 1 |
| 12 | [`scope::`](12_scope.md) | [`ScopeValue`](../type/07_scope_value.md) | varies | `local`, `relevant`, `under`, `global`, `around` | Session/project discovery scope | 8 |
| 13 | [`session::`](13_session.md) | [`SessionFilter`](../type/08_session_filter.md) / [`SessionId`](../type/09_session_id.md) | — | ID substring or exact | Session filter or scope pin | 4 |
| 14 | [`session_id::`](14_session_id.md) | [`SessionId`](../type/09_session_id.md) | — | exact session ID | Direct session identifier | 2 |
| 15 | [`show_sessions::`](15_sessions.md) | Boolean | `0` | `0`, `1` | Explicit session display toggle | 1 |
| 16 | [`target::`](16_target.md) | [`TargetType`](../type/11_target_type.md) | `projects` | `projects`, `sessions`, `entries`, `conversations` | Count operation target | 1 |
| 17 | [`topic::`](17_topic.md) | [`TopicName`](../type/12_topic_name.md) | — | identifier string | Session topic suffix | 5 |
| 18 | [`type::`](18_type.md) | [`ProjectType`](../type/06_project_type.md) | `all` | `uuid`, `path`, `all` | Project naming scheme filter | 1 |
| 19 | [`show_stat::`](19_show_stat.md) | Boolean | `0` | `0`, `1` | Session statistics footer in content mode | 1 |
| 20 | [`strategy::`](20_strategy.md) | [`StrategyType`](../type/13_strategy_type.md) | auto-detect | `resume`, `fresh` | Resume strategy override | 1 |
| 21 | [`count::`](21_count.md) | Boolean | `0` | `0`, `1` | Output count only flag (with `ids::1`) | 1 |
| 22 | [`limit::`](22_limit.md) | Integer | `0` | Integer ≥ 0 | Row display cap (per-project, flat, or grouped, command-dependent) | 3 |
| 23 | [`show_tokens::`](23_show_tokens.md) | Boolean | `0` | `0`, `1` | Token usage section in output | 2 |
| 24 | [`show_tree::`](24_show_tree.md) | Boolean | `0` | `0`, `1` | Agent tree-indented display | 1 |
| 25 | [`last::`](25_last.md) | Integer | varies | Integer ≥ 0 | Number of trailing entries to print | 2 |
| 26 | [`depth::`](26_depth.md) | Integer | `3` | Integer ≥ 0 | Path-component depth cap for scope walks | 2 |
| 27 | [`since_days::`](27_since_days.md) | Integer | — | Integer ≥ 0 | Recency window in days (`0` = last 24 hours) | 1 |
| 28 | [`show_topic::`](28_show_topic.md) | Boolean | `0` | `0`, `1` | First user message text on session lines | 1 |
| 29 | [`filter::`](29_filter.md) | [`PathSubstring`](../type/04_path_substring.md) | — | any string | Path-substring filter on resolved projects | 1 |
| 30 | [`detail::`](30_detail.md) | `DetailLevel` | varies | `projects`, `sessions` | Output detail level | 2 |
| 31 | [`ids::`](31_ids.md) | Boolean | `0` | `0`, `1` | Raw conversation-ID scripting output | 1 |
| 32 | [`fields::`](32_fields.md) | [`FieldSelector`](../type/15_field_selector.md) | — | 18 field names, or `all` | Attribute-projection field selector | 1 |
| 33 | [`index::`](33_index.md) | Integer | — | Integer ≥ 1 | Single-message position selector | 1 |
| 34 | [`group::`](34_group.md) | String enum | `session` | `session`, `project`, `model`, `day` | Aggregation dimension for `.rollup` | 1 |
| 35 | [`sort::`](35_sort.md) | String enum | `total` | `total`, `input`, `output`, `cache`, `max_context`, `calls`, `sessions`, `group` | Sort column for `.rollup`'s grouped rows | 1 |
| 36 | [`order::`](36_order.md) | String enum | `desc` | `asc`, `desc` | Sort direction for `.rollup` | 1 |
| 37 | [`model::`](37_model.md) | String | — | any string | Model-name substring filter for `.rollup` | 1 |
| 38 | [`columns::`](38_columns.md) | String (comma list) | 9-column default | 11 valid keys | Column projection for `.rollup` | 1 |
| 39 | [`session_ids::`](39_session_ids.md) | String (comma list) | — | session IDs or unique prefixes | Cross-project conversation selector for `.cost` | 1 |
| 40 | [`agents::`](40_agents.md) | Boolean | `1` | `0`, `1` | Agent fold-in toggle for `.cost` | 1 |

**Total:** 40 parameters, all implemented across their specified commands — including `depth::` (row 26) for `.usage` and `.rollup` (see `../command/13_usage.md`, `../command/14_rollup.md`); `filter::`, `detail::`, `ids::` (rows 29-31) for `.projects`' absorption of `.list` (see `../command/07_projects.md` and `../command_group/readme.md § Command Removal: .list -> .projects`); `last::`/`detail::` (rows 25, 30) for `.show`'s project-overview branch (see `../command/03_show.md`); `fields::`/`index::` (rows 32-33) for `.show`'s attribute-projection and single-message selection (see `../command/03_show.md`); `group::`/`sort::`/`order::`/`model::`/`columns::` (rows 34-38), introduced entirely for `.rollup` (see `../command/14_rollup.md`); and `session_ids::`/`agents::` (rows 39-40), introduced entirely for `.cost` (see `../command/15_cost.md` — `agents::` is deliberately distinct from row 1's `agent::` session-type filter, see `40_agents.md`).
