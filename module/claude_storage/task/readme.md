# task

## Tasks Index

| Order | ID | Advisability | Value | Easiness | Safety | Priority | Status | Executor | Task | Purpose |
|-------|----|-------------|-------|----------|--------|----------|--------|----------|------|---------|
| 1 | 029 | 0 | 7 | 6 | 5 | 2 | ✅ (Completed) | any | [Rename `.path` → `.project.path` and `.exists` → `.project.exists`](completed/029_rename_path_exists_to_project_namespace.md) | Align command names with entity taxonomy; update YAML, routing, Rust fns, and tests |
| 2 | 030 | 0 | 5 | 9 | 9 | 2 | ✅ (Completed) | any | [Fix issue-028 ID collision — rename cwd-default markers to issue-037](completed/030_fix_issue028_collision.md) | Rename `bug_reproducer(issue-028)` to `issue-037` in tests; add `Fix(issue-037)` source comment |
| 3 | 028 | 0 | 8 | 8 | 8 | 3 | ✅ (Completed) | any | [Fix `.session.dir` and `.session.ensure` — default `path::` to cwd](completed/028_session_dir_cwd_default.md) | Replace `ok_or_else` guard with `resolve_cmd_path` so bare invocation defaults to cwd |
| 4 | 007 | 0 | 8 | 5 | 8 | 0 | ✅ (Completed) | any | [Active session summary as default output](completed/007_sessions_active_summary_default.md) | Show most-recent session details instead of session list on bare invocation |
| 4 | 002 | 0 | 8 | 4 | 7 | 0 | ✅ (Completed) | any | [Session family tree detection](completed/002_session_families.md) | Implement hierarchical display grouping agent sessions under their parent |
| 5 | 008 | 0 | 7 | 8 | 9 | 0 | ✅ (Completed) | any | [Update CLI test docs — .sessions summary mode](completed/008_cli_test_docs_sessions_summary.md) | Rewrite IT-1 and add IT-30..IT-35 for summary behavior; update EC-7 |
| 6 | 009 | 0 | 7 | 8 | 9 | 0 | ✅ (Completed) | any | [Update CLI ref docs — .sessions summary mode](completed/009_cli_ref_docs_sessions_summary.md) | commands.md summary section + scope default fix + YAML + readme update |
| 7 | 003 | 0 | 8 | 7 | 8 | 0 | ✅ (Completed) | any | [Change .sessions default scope to under (impl)](completed/003_sessions_default_scope_impl.md) | Code + YAML + test changes for scope::under default |
| 8 | 001 | 0 | 6 | 8 | 8 | 0 | ✅ (Completed) | any | [Issue: entries param ignored without session_id](completed/001_issue.md) | Fix entries parameter silently ignored when session_id absent |
| 9 | 006 | 0 | 9 | 9 | 9 | 0 | ✅ (Completed) | any | [Install clg binary with scope::under default](completed/006_install_clg_binary.md) | Rebuild and install binary so scope default change takes effect |
| 10 | 004 | 0 | 6 | 9 | 9 | 0 | ✅ (Completed) | any | [Update spec.md — .sessions scope default](completed/004_sessions_default_scope_spec.md) | Spec update for scope::under default |
| 11 | 005 | 0 | 6 | 9 | 9 | 0 | ✅ (Completed) | any | [Update CLI test docs — .sessions scope default](completed/005_sessions_default_scope_cli_docs.md) | CLI test docs IT-1 and CD-2 updated for under default |
| 12 | 010 | 0 | 7 | 9 | 9 | 0 | ✅ (Completed) | any | [Fix docs/ consistency](completed/010_docs_consistency.md) | Fix stale counts, isolated graph components, readme accuracy |
| 13 | 011 | 0 | 7 | 6 | 9 | 0 | ✅ (Completed) | any | [Create docs/cli/format/ catalog](completed/011_cli_format_catalog.md) | CLI Output Format Doc Entity for export rendering modes |
| 14 | 012 | 0 | 8 | 9 | 9 | 0 | ✅ (Completed) | any | [Fix .sessions is_default verbosity guard](completed/012_fix_is_default_verbosity_sessions.md) | Remove verbosity from is_default so verbosity::1 stays in summary mode |
| 15 | 013 | 0 | 5 | 9 | 9 | 0 | ✅ (Completed) | any | [Remove deprecated .show.project command](completed/013_remove_show_project_command.md) | Delete dead deprecated stub and its test infrastructure |
| 16 | 014 | 0 | 7 | 8 | 8 | 0 | ✅ (Completed) | any | [Remove duplicate .session command](completed/014_remove_session_command.md) | Deduplicate with .exists; eliminate tab-completion hazard |
| 17 | 015 | 0 | 8 | 6 | 6 | 0 | ✅ (Completed) | any | [Rename .sessions → .projects](completed/015_rename_sessions_to_projects.md) | Align command name with user-facing concept (projects, not sessions) |
| 18 | 016 | 0 | 9 | 3 | 5 | 0 | ✅ (Completed) | any | [Redesign .projects output as project-centric](completed/016_redesign_projects_output.md) | Show project summaries (aggregated) instead of session UUID lists |
| 19 | 017 | 0 | 7 | 9 | 5 | 0 | ✅ (Completed) | any | [Fix stale "Active session" mode-boundary assertions](completed/017_fix_stale_active_session_assertions.md) | Replace obsolete pre-016 marker in IT-34/IT-35; remove redundant checks from IT-1/IT-47 |
| 20 | 018 | 1008 | 8 | 7 | 9 | 2 | ✅ (Completed) | any | [Implement `scope::around` for `.projects`](completed/018_implement_scope_around.md) | Add bidirectional neighborhood scope and make it the default for `.projects` |
| 21 | 019 | 1512 | 9 | 7 | 8 | 3 | ✅ (Completed) | any | [Remove `.projects` summary mode](completed/019_remove_summary_mode.md) | Delete `is_default` gate + `render_active_project_summary`; bare invocation becomes list mode |
| 22 | 021 | 0 | 9 | 5 | 7 | 4 | ✅ (Completed) | any | [Introduce `Conversation` type and chain detection](completed/021_conversation_type_and_chain_detection.md) | Add `struct Conversation`, `group_into_conversations`, refactor projects rendering to iterate `Vec<Conversation>` |
| 23 | 022 | 0 | 8 | 7 | 7 | 3 | ✅ (Completed) | any | [Update `.projects` output to use conversation terminology](completed/022_projects_show_conversations.md) | Replace `(N sessions)` with `(N conversations)` unconditionally; prerequisite: task 021 |
| 24 | 020 | 0 | 6 | 10 | 9 | 2 | ✅ (Completed) | any | [Make `cli` feature default in `Cargo.toml`](completed/020_cli_feature_default.md) | Change `default = []` to `default = ["cli"]` so `cargo install` builds binaries |
| 25 | 024 | 0 | 7 | 8 | 9 | 2 | ✅ (Completed) | any | [Add B17 and B18 behavior validation tests](completed/024_behavior_b17_b18_tests.md) | Validate parentUuid self-containment and null-first-entry invariants against real storage |
| 26 | 023 | 0 | 8 | 5 | 7 | 3 | ✅ (Completed) | any | [Extend `.list` and `.count` with `type::conversation`](completed/023_list_count_conversation_type.md) | Add conversation listing and count mode; prerequisite: tasks 021+022 |
| 27 | 025 | 0 | 9 | 8 | 9 | 0 | ✅ (Completed) | any | [Fix project display path — always show topic regardless of filesystem state](completed/025_fix_display_path_topic_existence_check.md) | Remove `candidate.exists()` guard in `decode_project_display`; sessions always attributed to their actual CWD |
| 28 | 027 | 0 | 9 | 7 | 8 | 0 | ✅ (Completed) | any | [Fix `.show session_id::` — search topic project directories](completed/027_fix_show_session_topic_dirs.md) | Replace `load_project_for_cwd()` with scope::local scan so sessions in `--commit`/`--default-topic` dirs are found |
| 29 | 026 | 0 | 7 | 7 | 8 | 0 | ✅ (Completed) | any | [Refactor storage key parsing — extract shared utilities](completed/026_refactor_storage_key_parsing_utilities.md) | Extract `split_storage_key`, `strip_topic_suffix`, `matches_under`, `matches_relevant`, `decode_storage_base`, `topic_to_dir`; eliminate 4× inline duplication and `around` copy-paste |
