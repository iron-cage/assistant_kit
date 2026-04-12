# Task Registry — agent_kit

| Dir | Responsibility |
|-----|----------------|
| completed/ | Completed tasks archived after closure |
| backlog/ | Backlog tasks deferred for future planning |
| cancelled/ | Cancelled tasks archived after cancellation |

## Active Tasks

| ID | Status | Title | Category | Created | File |
|---|---|---|---|---|---|
| 100 | 🎯 Available | Fix settings_get_routine JSON output losing type information | bug | 2026-04-12 | [100](100_fix_settings_get_json_type_loss.md) |
| 101 | 🎯 Available | Fix processes_kill_routine swallowing signal errors and never producing exit 2 | bug | 2026-04-12 | [101](101_fix_processes_kill_silent_signal_errors.md) |
| 095 | 🎯 Available | Fix require_claude_paths producing identical errors for distinct failure conditions | bug | 2026-04-12 | [095](095_fix_require_claude_paths_error_messages.md) |
| 099 | 🎯 Available | Fix dead output-control params in version_install, version_guard, and processes_kill | bug | 2026-04-12 | [099](099_fix_dead_output_control_params.md) |
| 096 | 🎯 Available | Handle UTF-16 surrogate pairs in parse_json_string_value | bug | 2026-04-12 | [096](096_fix_surrogate_pairs_in_json_string_parser.md) |
| 097 | 🎯 Available | Fix extract_releases fragile literal-split tag parsing | quality | 2026-04-12 | [097](097_fix_extract_releases_fragile_tag_parsing.md) |
| 098 | 🎯 Available | Minor code quality fixes: rename chrono_timestamp and fix status alignment | quality | 2026-04-12 | [098](098_minor_code_quality_fixes.md) |
| 102 | 📥 Backlog | Split `claude_storage/src/cli/mod.rs` into per-command modules | refactoring | 2026-04-12 | [102](backlog/102_split_claude_storage_cli_mod.md) |
| 103 | 📥 Backlog | Split `sessions_command_test.rs` into focused test files | quality | 2026-04-12 | [103](backlog/103_split_sessions_command_test.md) |
| 104 | 📥 Backlog | Split `claude_runner_core/src/command.rs` into parameter-group modules | refactoring | 2026-04-12 | [104](backlog/104_split_claude_runner_core_command.md) |
| 105 | ✅ Complete | Create `docs/` structure for claude_assets, claude_assets_core, claude_tools | documentation | 2026-04-12 | [105](completed/105_create_docs_for_new_crates.md) |
| 091 | ✅ Complete | Change ultrathink injection from prefix to suffix (`\n\nultrathink`) | bug | 2026-04-11 | [091](completed/091_ultrathink_suffix_injection.md) |
| 090 | ✅ Complete | Implement ultrathink default message prefix in `clr` binary | feature | 2026-04-11 | [090](completed/090_ultrathink_default_prefix_impl.md) |
| 089 | ✅ Complete | Create `claude_assets` crate pair — multi-artifact installer CLI (`cla`) | feature | 2026-04-11 | [089](completed/089_claude_assets_installer.md) |
| 087 | ✅ Complete | Introduce `agent_kit` aggregation crate (Layer 2 facade) | architecture | 2026-04-11 | [087](completed/087_introduce_agent_kit_crate.md) |
| 086 | ✅ Complete | Implement `.account.limits` command (FR-18) | feature | 2026-04-07 | [086](completed/086_implement_account_limits.md) |
| 085 | ✅ Complete | Update claude_runner `docs/cli/` — `--system-prompt` and `--append-system-prompt` | documentation | 2026-04-06 | [085](completed/085_runner_cli_docs_system_prompt.md) |
| 084 | ✅ Complete | Update claude_runner `spec.md` — `--system-prompt` and `--append-system-prompt` | documentation | 2026-04-06 | [084](completed/084_runner_spec_system_prompt.md) |
| 083 | ✅ Complete | Update claude_runner docs/cli/ — `--trace` flag + stale content fixes | documentation | 2026-04-04 | [083](completed/083_runner_cli_docs_trace_flag.md) |
| 082 | ✅ Complete | Update claude_runner spec.md — `--trace` flag | documentation | 2026-04-04 | [082](completed/082_runner_spec_trace_flag.md) |
| 080 | ✅ Complete | Add `.credentials.status` command — live credentials vs account store | feature | 2026-04-04 | [080](completed/080_credentials_status_command.md) |
| 071 | ✅ Complete | Add dry-run mode and compact inspection to `ClaudeCommand` | feature | 2026-04-04 | [071](../module/claude_runner_core/task/completed/071_claude_runner_core_dry_run_and_inspect.md) |
| 072 | ✅ Complete | Add typed builder methods for I/O parameters | feature | 2026-04-04 | [072](../module/claude_runner_core/task/completed/072_claude_runner_core_io_params.md) |
| 073 | ✅ Complete | Add typed builder methods for tool and directory control | feature | 2026-04-04 | [073](../module/claude_runner_core/task/completed/073_claude_runner_core_tool_dir_params.md) |
| 074 | ✅ Complete | Add typed builder methods for session management | feature | 2026-04-04 | [074](../module/claude_runner_core/task/completed/074_claude_runner_core_session_params.md) |
| 075 | ✅ Complete | Add typed builder methods for system prompt and permissions | feature | 2026-04-04 | [075](../module/claude_runner_core/task/completed/075_claude_runner_core_prompt_permission_params.md) |
| 076 | ✅ Complete | Add typed builder methods for model and budget control | feature | 2026-04-04 | [076](../module/claude_runner_core/task/completed/076_claude_runner_core_model_budget_params.md) |
| 077 | ✅ Complete | Add typed builder methods for MCP and extensions | feature | 2026-04-04 | [077](../module/claude_runner_core/task/completed/077_claude_runner_core_mcp_extension_params.md) |
| 078 | ✅ Complete | Add typed builder methods for debug and advanced CLI | feature | 2026-04-04 | [078](../module/claude_runner_core/task/completed/078_claude_runner_core_debug_advanced_params.md) |
| 079 | ✅ Complete | Add typed builder methods for terminal and IDE integration | feature | 2026-04-04 | [079](../module/claude_runner_core/task/completed/079_claude_runner_core_terminal_ide_params.md) |
| 068 | ✅ Complete | Add session path and lifecycle commands to `claude_storage` | feature | 2026-04-02 | [068](completed/068_claude_storage_session_path_commands.md) |
| 069 | ✅ Complete | Update spec.md: Entry/API Message data model + `.show` cross-project behavior | documentation | 2026-04-02 | [069](completed/069_spec_show_cross_project_and_data_model.md) |
| 070 | ✅ Complete | Update CLI docs: `.show` cross-project behavior + Scope Configuration group note | documentation | 2026-04-02 | [070](completed/070_cli_docs_show_cross_project.md) |
| 067 | ✅ Complete | Fix `.account.status` v::1 Sub/Tier/Email/Org spec compliance | spec-compliance | 2026-03-31 | [067](completed/067_account_status_v1_sub_tier.md) |
| 066 | ✅ Complete | Update CLI docs for `.account.status name::` extension | documentation | 2026-03-31 | [066](completed/066_cli_docs_account_status_name.md) |
| 065 | ✅ Complete | TDD implementation of FR-16 — `.account.status name::` parameter | feature | 2026-03-31 | [065](completed/065_implement_account_status_name.md) |
| 064 | ✅ Complete | Add Known Pitfalls section to src/persist.rs module doc | documentation | 2026-03-31 | [064](completed/064_persist_known_pitfalls.md) |
| 063 | ✅ Complete | Update spec.md — FR-16 `.account.status name::` extension | documentation | 2026-03-31 | [063](completed/063_spec_account_status_fr16.md) |
| 062 | ✅ Complete | Update docs/cli/commands.md — issue-030 path display fix note | documentation | 2026-03-31 | [062](completed/062_sessions_cli_docs_update_issue030.md) |
| 061 | ✅ Complete | Update spec.md — issue-030 path display and scope::under encoding constraint | documentation | 2026-03-31 | [061](completed/061_sessions_spec_update_issue030_scope_under.md) |
| 060 | ✅ Complete | Fix scope::under false positive — sibling modules with underscore names | bug | 2026-03-31 | [060](completed/060_sessions_scope_under_encoding_ambiguity.md) |
| 059 | ✅ Complete | Create spec.md and readme.md for claude_session crate | documentation | 2026-03-31 | [059](completed/059_claude_session_docs.md) |
| 058 | ✅ Complete | Implement default-on skip-permissions in clr binary | feature | 2026-03-31 | [058](completed/058_clr_default_skip_permissions_impl.md) |
| 057 | ✅ Complete | Update claude_runner CLI docs for default-on skip-permissions | documentation | 2026-03-31 | [057](completed/057_runner_cli_docs_default_flags.md) |
| 056 | ✅ Complete | Add Default Flags Principle to claude_runner spec.md | documentation | 2026-03-31 | [056](completed/056_runner_spec_default_flags.md) |
| 055 | ✅ Complete | Aggregate claude_profile into claude_tools super-app | architecture | 2026-03-29 | [055](completed/055_claude_tools_full_aggregation.md) |
| 054 | ✅ Complete | Migrate claude_profile to reusable unilang integration | refactoring | 2026-03-29 | [054](completed/054_claude_profile_unilang_integration.md) |
| 053 | ✅ Complete | Migrate claude_manager to unilang YAML command definitions | refactoring | 2026-03-29 | [053](completed/053_claude_manager_unilang_yaml_migration.md) |
| 052 | ✅ Complete | Bug fix issue-028: `.show` header and `.show.project` list show "(1 entries)" — wrong plural for irregular noun | bug | 2026-03-29 | — |
| 051 | ✅ Complete | Bug fix issue-027: `.list sessions::1` shows "(1 sessions)" — wrong plural in per-project label | bug | 2026-03-29 | — |
| 050 | ✅ Complete | Bug fix issue-026: `.export` IO error loses output path context ("unknown operation") | bug | 2026-03-29 | — |
| 049 | ✅ Complete | Bug fix issue-025: "Found 1 sessions/projects/matches:" — wrong plural when count == 1 | bug | 2026-03-29 | — |
| 048 | ✅ Complete | Bug fix issue-024: `.sessions scope::local/relevant/under` returns 0 when path has underscores | bug | 2026-03-29 | — |
| 047 | ✅ Complete | Implement `.sessions` command — scope-aware session listing | feature | 2026-03-28 | [047](completed/047_implement_sessions_command.md) |
| 046 | ✅ Complete | Fix `.version.guard` watch loop exits on install error instead of continuing | bug | 2026-03-28 | [046](completed/046_fix_watch_loop_error_exit.md) |
| 045 | ✅ Complete | Four-layer crate architecture (claude_common + 2 Layer 1 cores + clt super-app) | architecture | 2026-03-28 | [045](completed/045_four_layer_crate_architecture.md) |
| 044 | ✅ Complete | clr: print default when message given + `--interactive` flag | feature | 2026-03-28 | [044](completed/044_clr_print_default_and_interactive_flag.md) |
| 036 | ✅ Complete | Rename binary to `cm` and fix `cm .` showing help | usability | 2026-03-24 | — |
| 037 | ✅ Complete | Move process management to claude_runner_core | refactoring | 2026-03-24 | — |
| 038 | ✅ Complete | Deduplicate account handlers — claude_profile owns account management | refactoring | 2026-03-24 | — |
| 039 | ✅ Complete | Add `version::` parameter to `.version.guard` | feature | 2026-03-24 | — |
| 040 | ✅ Complete | Sync docs with unilang 5-phase pipeline migration (7 files updated) | documentation | 2026-03-24 | — |
| 041 | ✅ Complete | Rename crate claude_session → claude_profile (79 files updated) | refactoring | 2026-03-28 | — |
| 042 | ✅ Complete | Implement FR-13 auto_rotate() — one-call best-account rotation | feature | 2026-03-28 | — |
| 043 | ✅ Complete | Rename workspace identity claude_tools → agent_kit (12 files, ~23 occurrences) | refactoring | 2026-03-28 | — |

## Completed Tasks (Legacy Index)

| ID | Status | Title | Category | Created |
|---|---|---|---|---|
| 027 | ✅ Complete | Fix #[cfg(test)] violations — move inline tests to tests/ in claude_profile | compliance | 2026-03-21 |
| 028 | ✅ Complete | Fix claude_profile test codestyle: responsibility test new-line braces | quality | 2026-03-21 |
| 029 | ✅ Complete | Create docs/cli/parameter_interactions.md (L4 completeness blocker) | documentation | 2026-03-21 |
| 030 | ✅ Complete | Fix docs/cli types.md sequential headers, Methods sections, parameter_groups.md Why NOT | documentation | 2026-03-21 |
| 031 | ✅ Complete | Redesign CLI to --flag value syntax, remove unilang dependency | feature | 2026-03-21 |
| 032 | ✅ Complete | Add --verbosity flag to claude_runner (VerbosityLevel type + impl + docs) | feature | 2026-03-21 |
| 033 | ✅ Complete | Move continuation detection from claude_profile to claude_storage_core | refactoring | 2026-03-21 |
| 034 | ✅ Complete | Move SessionManager from claude_profile to claude_runner_core | refactoring | 2026-03-21 |
| 035 | ✅ Complete | Add .session command to claude_storage CLI (check_continuation API surface) | feature | 2026-03-21 |
| 001 | ✅ Complete | Migrate claude_storage_core from wtools to claude_tools | infrastructure | 2026-03-13 |
| 002 | ✅ Complete | Migrate claude_storage from wtools to claude_tools | infrastructure | 2026-03-13 |
| 003 | ✅ Complete | Migrate claude_profile from wtools to claude_tools | infrastructure | 2026-03-13 |
| 004 | ✅ Complete | Migrate claude_runner_core from wtools to claude_tools | infrastructure | 2026-03-13 |
| 005 | ✅ Complete | Migrate claude_runner (was claude_runner_cli) from wtools to claude_tools | infrastructure | 2026-03-13 |
| 006 | ✅ Complete | Move claude_runner from wtools to consumer workspace (path deps → workspace deps) | infrastructure | 2026-03-13 |
| 007 | ✅ Complete | Unify all crate versions to 1.0.0 via workspace.package.version | infrastructure | 2026-03-13 |
| 008 | ✅ Complete | Remove 6 crate directories from wtools | cleanup | 2026-03-13 |
| 009 | ✅ Complete | Verify 542/542 tests pass in claude_tools | validation | 2026-03-13 |
| 010 | ✅ Complete | Create workspace-level spec.md | specification | 2026-03-13 |
| 011 | ✅ Complete | Fix stale workspace references in 3 docs | documentation | 2026-03-13 |
| 012 | ✅ Complete | Complete claude_runner/spec.md in consumer workspace (89 → 303 lines) | specification | 2026-03-13 |
| 013 | ✅ Complete | Add .gitignore (with Cargo.lock policy for binary crates) | infrastructure | 2026-03-13 |
| 014 | ✅ Complete | Add cfg_attr doc inclusion to 4 library crates | documentation | 2026-03-13 |
| 015 | ✅ Complete | Run full L3 verification (nextest + doc test + clippy) | validation | 2026-03-13 |
| 016 | ✅ Complete | Fix 3 organizational violations (missing module/readme.md and docs/ readme.md) | documentation | 2026-03-13 |
| 017 | ✅ Complete | Create docs/cli.md user-facing CLI reference for .claude commands | documentation | 2026-03-13 |
| 018 | ✅ Complete | Add claude_runner to consumer workspace crates.md (alphabetical) | documentation | 2026-03-13 |
| 019 | ✅ Complete | Fix clippy missing_inline_in_public_items across claude_storage_core and claude_storage | quality | 2026-03-13 |
| 020 | ✅ Complete | Fix org violations: add readme.md to testing/param/ (9 files) and testing/param_group/ (4 files) | documentation | 2026-03-13 |
| 021 | ✅ Complete | Bug fix issue-015: .status performance — add global_stats_fast() O(P+S) fast path | bug | 2026-03-13 |
| 022 | ✅ Complete | Bug fix issue-016: count_entries() counted metadata lines, not just user/assistant | bug | 2026-03-13 |
| 023 | ✅ Complete | Bug fix issue-017: .count failed on projects with corrupted JSONL sessions | bug | 2026-03-13 |
| 024 | ✅ Complete | Bug fix issue-018: count_entries() full JSON parse caused .list min_entries hang | bug | 2026-03-13 |
| 025 | ✅ Complete | Bug fix issue-019: ExportFormat::from_str returned confusing I/O error for invalid format | bug | 2026-03-13 |
| 026 | ✅ Complete | Manual testing session: 5 bugs found/fixed, spec.md updated, tests/manual/readme.md updated | validation | 2026-03-13 |

## Metadata

**Status Distribution:**
- ✅ Complete: 92
- 🎯 Available: 7
- 📥 Backlog: 3

**Status Legend:**
- 📥 Backlog — not yet planned
- 🔄 Planned — scoped and ready
- ⏳ In Progress — actively being worked
- ✅ Complete — done and verified
- ⚠️ Superseded — replaced by another task
- ❌ Rejected — won't do

**Last Updated:** 2026-04-12 (TSK-092–098 created: claude_manager comprehensive audit — 7 bug/quality tasks available)
