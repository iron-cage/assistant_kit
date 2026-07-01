# claude_profile Tasks

<!-- task_system_metadata
type: root
registry_prefix: null
next_id: 6
-->

## Tasks Index

| Order | ID | Advisability | Value | Easiness | Safety | Priority | State | Executor | Dir | Task | Purpose |
|-------|----|-------------|-------|----------|--------|----------|-------|----------|-----|------|---------|
| 1 | 001 | 490 | 7 | 7 | 5 | 2 | ✅ (Completed) | any | tests/docs/ | [State Machine and Subprocess Test Spec Coverage](completed/001_state_machine_subprocess_spec_coverage.md) | Create tests/docs/state_machine/ and tests/docs/subprocess/ spec files |
| 2 | 002 | — | 8 | 6 | 5 | 2 | ✅ (Completed) | any | tests/docs/ | [Test Surface Remediation](completed/002_test_surface_remediation.md) | Remediate all audit findings: 3 missing surfaces, format violations, below-min counts, missing behavioral divergence |
| 3 | 003 | — | 7 | 8 | 5 | 2 | ✅ (Completed) | any | tests/cli/ | [verb::unclaim Test Implementation and assign BV-4 Gap Closure](completed/003_verb_unclaim_test_implementation.md) | Implement REMOVED_TOGGLE BV-4 tests for verb::assign and verb::unclaim; fix stale FT-02 comment |
| 4 | 004 | — | 6 | 8 | 5 | 2 | 🎯 (Verified) | any | tests/cli/ | [Add IT-N Spec Cross-References to CLI Test File Matrices](004_cli_l5_spec_crossref.md) | Link IT-N/EC-N spec IDs to implementing test functions for 6 spec files updated in the L5 normalization session |
| 5 | 005 | — | 9 | 6 | 5 | 1 | 🎯 (Verified) | any | src/ | [JSON Config Loading — Implementation](005_json_config_loading.md) | Implement --args-file / CLR_ARGS_FILE / stdin JSON pipe for all executing subcommands with CLI > JSON > CLR_* > defaults precedence |
