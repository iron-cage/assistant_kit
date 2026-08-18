# Parameters

### Scope

- **Purpose**: Document individual parameter specifications for the `claude_storage` CLI.
- **Responsibility**: Per-parameter detail pages with type, defaults, and command cross-refs.
- **In Scope**: All 31 CLI parameters with type constraints, defaults, valid values, and command usage.
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
| `25_tail.md` | tail:: — trailing entry count for .tail and .show's project overview |
| `26_depth.md` | depth:: — path-component depth cap for .usage scope walks |
| `27_since_days.md` | since_days:: — recency window in days for .projects |
| `28_show_topic.md` | show_topic:: — first user message on session lines toggle |
| `29_filter.md` | filter:: — path-substring filter on resolved projects for .projects |
| `30_detail.md` | detail:: — output detail level (projects vs sessions) for .projects and .show |
| `31_ids.md` | ids:: — raw conversation-ID scripting output toggle for .projects |

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
| 9 | [`path::`](09_path.md) | varies | varies | varies | Path argument (semantics vary by command) | 13 |
| 10 | [`project::`](10_project.md) | [`ProjectId`](../type/05_project_id.md) | current dir | path, uuid, substring | Project scope identifier | 5 |
| 11 | [`query::`](11_query.md) | String | — | any string | Search query string | 1 |
| 12 | [`scope::`](12_scope.md) | [`ScopeValue`](../type/07_scope_value.md) | varies | `local`, `relevant`, `under`, `global`, `around` | Session/project discovery scope | 7 |
| 13 | [`session::`](13_session.md) | [`SessionFilter`](../type/08_session_filter.md) / [`SessionId`](../type/09_session_id.md) | — | ID substring or exact | Session filter or scope pin | 4 |
| 14 | [`session_id::`](14_session_id.md) | [`SessionId`](../type/09_session_id.md) | — | exact session ID | Direct session identifier | 2 |
| 15 | [`show_sessions::`](15_sessions.md) | Boolean | `0` | `0`, `1` | Explicit session display toggle | 1 |
| 16 | [`target::`](16_target.md) | [`TargetType`](../type/11_target_type.md) | `projects` | `projects`, `sessions`, `entries`, `conversations` | Count operation target | 1 |
| 17 | [`topic::`](17_topic.md) | [`TopicName`](../type/12_topic_name.md) | — | identifier string | Session topic suffix | 5 |
| 18 | [`type::`](18_type.md) | [`ProjectType`](../type/06_project_type.md) | `all` | `uuid`, `path`, `all` | Project naming scheme filter | 1 |
| 19 | [`show_stat::`](19_show_stat.md) | Boolean | `0` | `0`, `1` | Session statistics footer in content mode | 1 |
| 20 | [`strategy::`](20_strategy.md) | [`StrategyType`](../type/13_strategy_type.md) | auto-detect | `resume`, `fresh` | Resume strategy override | 1 |
| 21 | [`count::`](21_count.md) | Boolean | `0` | `0`, `1` | Output count only flag (with `ids::1`) | 1 |
| 22 | [`limit::`](22_limit.md) | Integer | `0` | Integer ≥ 0 | Session display cap (per-project or flat, command-dependent) | 2 |
| 23 | [`show_tokens::`](23_show_tokens.md) | Boolean | `0` | `0`, `1` | Token usage section in output | 2 |
| 24 | [`show_tree::`](24_show_tree.md) | Boolean | `0` | `0`, `1` | Agent tree-indented display | 1 |
| 25 | [`tail::`](25_tail.md) | Integer | varies | Integer ≥ 0 | Number of trailing entries to print | 2 |
| 26 | [`depth::`](26_depth.md) | Integer | `3` | Integer ≥ 0 | Path-component depth cap for scope walks | 1 |
| 27 | [`since_days::`](27_since_days.md) | Integer | — | Integer ≥ 0 | Recency window in days (`0` = last 24 hours) | 1 |
| 28 | [`show_topic::`](28_show_topic.md) | Boolean | `0` | `0`, `1` | First user message text on session lines | 1 |
| 29 | [`filter::`](29_filter.md) | [`PathSubstring`](../type/04_path_substring.md) | — | any string | Path-substring filter on resolved projects | 1 |
| 30 | [`detail::`](30_detail.md) | `DetailLevel` | varies | `projects`, `sessions` | Output detail level | 2 |
| 31 | [`ids::`](31_ids.md) | Boolean | `0` | `0`, `1` | Raw conversation-ID scripting output | 1 |

**Total:** 31 parameters. 28 are implemented across some or all of their specified commands — including `depth::` (row 26) for `.usage`, see `../command/13_usage.md`. 3 are not yet implemented anywhere — `filter::`, `detail::`, `ids::` (rows 29-31) — specified for the not-yet-implemented absorption of `.list` into `.projects`, see `../command/07_projects.md` and `../command_group/readme.md § Command Removal: .list -> .projects`. Two parameters are partially implemented (real on one command, spec-only on another): `tail::` (row 25) is implemented for `.tail` but not yet for `.show`'s project-overview branch; `detail::` (row 30) is additionally specified — and, like its `.projects` usage, not yet implemented — for that same `.show` branch. See `../command/03_show.md`.
