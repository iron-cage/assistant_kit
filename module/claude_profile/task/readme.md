# claude_profile Tasks

<!-- task_system_metadata
type: root
registry_prefix: null
next_id: 10
-->

## Tasks Index

| Order | ID | Advisability | Value | Easiness | Safety | Priority | State | Executor | Dir | Task | Purpose |
|-------|----|-------------|-------|----------|--------|----------|-------|----------|-----|------|---------|
| 1 | 001 | 490 | 7 | 7 | 5 | 2 | ✅ (Completed) | any | tests/docs/ | [State Machine and Subprocess Test Spec Coverage](completed/001_state_machine_subprocess_spec_coverage.md) | Create tests/docs/state_machine/ and tests/docs/subprocess/ spec files |
| 2 | 002 | — | 8 | 6 | 5 | 2 | ✅ (Completed) | any | tests/docs/ | [Test Surface Remediation](completed/002_test_surface_remediation.md) | Remediate all audit findings: 3 missing surfaces, format violations, below-min counts, missing behavioral divergence |
| 3 | 003 | — | 7 | 8 | 5 | 2 | ✅ (Completed) | any | tests/cli/ | [verb::unclaim Test Implementation and assign BV-4 Gap Closure](completed/003_verb_unclaim_test_implementation.md) | Implement REMOVED_TOGGLE BV-4 tests for verb::assign and verb::unclaim; fix stale FT-02 comment |
| 4 | 004 | — | 6 | 8 | 5 | 2 | 🎯 (Verified) | any | tests/cli/ | [Add IT-N Spec Cross-References to CLI Test File Matrices](004_cli_l5_spec_crossref.md) | Link IT-N/EC-N spec IDs to implementing test functions for 6 spec files updated in the L5 normalization session |
| 5 | 005 | — | 9 | 6 | 5 | 1 | 🎯 (Verified) | any | src/ | [JSON Config Loading — Implementation](005_json_config_loading.md) | Implement --args-file / CLR_ARGS_FILE / stdin JSON pipe for all executing subcommands with CLI > JSON > CLR_* > defaults precedence |
| 6 | 006 | — | 8 | 7 | 5 | 1 | 🎯 (Verified) | any | module/claude_runner/tests/ | [--no-compact-window Test Coverage](006_no_compact_window_test_coverage.md) | Implement 12 #[test] functions covering CLAUDE_CODE_AUTO_COMPACT_WINDOW=200000 injection and --no-compact-window opt-out across all 4 running commands |
| 7 | 007 | — | 8 | 6 | 5 | 1 | 🎯 (Verified) | any | src/ + claude_quota/src/ | [Models List Command](007_models_list_command.md) | Implement `.models` command with offline/live modes, name filter, and three output formats; add fetch_models() and STATIC_MODELS to claude_quota |
| 8 | 008 | — | 9 | 6 | 5 | 1 | 🎯 (Verified) | any | src/ + module/claude_runner_core/src/ | [Model Select Command](008_model_select_command.md) | Implement `.model.select` command (get/set/reset modes) and ~/.clr/prefs.json subprocess model preference reader in claude_runner_core |
| 9 | 009 | — | 7 | 9 | 5 | 1 | 🎯 (Verified) | any | src/ | [Stale Model IDs Fix](009_stale_model_ids_fix.md) | Replace claude-opus-4-6 and claude-sonnet-4-6 with claude-opus-4-8 and claude-sonnet-5 in map_model_shorthand(), resolve_model(), registry descriptions, and test assertions |
