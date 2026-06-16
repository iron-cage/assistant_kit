# Doc Entities

## Master Doc Entities Table

| Type | Purpose | Master File | Instances |
|------|---------|-------------|----------:|
| `feature/` | Functional requirements for claude_profile capabilities | [readme.md](../feature/readme.md) | 37 |
| `cli/` (standalone) | Cross-cutting CLI reference (config params, dictionary, env params, interactions) | [cli/readme.md](../cli/readme.md) | 4 |
| `cli/command/` | CLI command specifications | [cli/command/readme.md](../cli/command/readme.md) | 7 |
| `cli/param/` | CLI parameter specifications | [cli/param/readme.md](../cli/param/readme.md) | 58 |
| `cli/param_group/` | CLI parameter group definitions | [cli/param_group/readme.md](../cli/param_group/readme.md) | 6 |
| `cli/type/` | CLI type definitions | [cli/type/readme.md](../cli/type/readme.md) | 4 |
| `cli/format/` | CLI output format specifications | [cli/format/readme.md](../cli/format/readme.md) | 3 |
| `cli/user_story/` | Canonical user stories mapping personas and goals to commands | [cli/user_story/readme.md](../cli/user_story/readme.md) | 5 |
| `cli/command_noun/` | Domain noun documentation (account, token, credentials) | [cli/command_noun/readme.md](../cli/command_noun/readme.md) | 3 |
| `cli/command_verb/` | Domain verb documentation (save, use, delete, limits, relogin, rotate, renewal, inspect, assign, status, unclaim) | [cli/command_verb/readme.md](../cli/command_verb/readme.md) | 11 |
| `invariant/` | Measurable constraints and architectural guarantees | [invariant/readme.md](../invariant/readme.md) | 6 |
| `research_interactive/` | Investigation findings on Claude binary behavior | [research_interactive/readme.md](../research_interactive/readme.md) | 1 |
| `tests/docs/cli/command/` | Per-command integration test case documentation | [tests/docs/cli/command/readme.md](../../tests/docs/cli/command/readme.md) | 19 |
| `tests/docs/cli/command_noun/` | Per-noun test case documentation | [tests/docs/cli/command_noun/readme.md](../../tests/docs/cli/command_noun/readme.md) | 3 |
| `tests/docs/cli/command_verb/` | Per-verb test case documentation | [tests/docs/cli/command_verb/readme.md](../../tests/docs/cli/command_verb/readme.md) | 11 |
| `tests/docs/cli/param/` | Per-parameter edge case test documentation | [tests/docs/cli/param/readme.md](../../tests/docs/cli/param/readme.md) | 55 |
| `tests/docs/cli/param_group/` | Per-group interaction test documentation | [tests/docs/cli/param_group/readme.md](../../tests/docs/cli/param_group/readme.md) | 6 |
| `tests/docs/cli/type/` | Per-type test case documentation | [tests/docs/cli/type/readme.md](../../tests/docs/cli/type/readme.md) | 4 |
| `tests/docs/cli/user_story/` | Per-story acceptance test documentation | [tests/docs/cli/user_story/readme.md](../../tests/docs/cli/user_story/readme.md) | 5 |
| `tests/docs/feature/` | Per-feature behavioral test documentation | [tests/docs/feature/readme.md](../../tests/docs/feature/readme.md) | 37 |
| `tests/docs/invariant/` | Per-invariant constraint test documentation | [tests/docs/invariant/readme.md](../../tests/docs/invariant/readme.md) | 6 |

## Master Doc Instances Table

| Entity | ID | Name | File |
|--------|----|------|------|
| feature | 001 | Account Store Initialization | [feature/001_account_store_init.md](../feature/001_account_store_init.md) |
| feature | 002 | Save Account | [feature/002_account_save.md](../feature/002_account_save.md) |
| feature | 003 | List Accounts | [feature/003_account_list.md](../feature/003_account_list.md) |
| feature | 004 | Switch Account | [feature/004_account_use.md](../feature/004_account_use.md) |
| feature | 005 | Delete Account | [feature/005_account_delete.md](../feature/005_account_delete.md) |
| feature | 006 | Token Status | [feature/006_token_status.md](../feature/006_token_status.md) |
| feature | 007 | File Topology | [feature/007_file_topology.md](../feature/007_file_topology.md) |
| feature | 008 | Auto Rotate | [feature/008_auto_rotate.md](../feature/008_auto_rotate.md) |
| feature | 009 | Token Usage Reporting | [feature/009_token_usage.md](../feature/009_token_usage.md) |
| feature | 010 | Persistent Storage Path | [feature/010_persistent_storage.md](../feature/010_persistent_storage.md) |
| feature | 011 | Account Status by Name | [feature/011_account_status_by_name.md](../feature/011_account_status_by_name.md) |
| feature | 012 | Live Credentials Status | [feature/012_live_credentials_status.md](../feature/012_live_credentials_status.md) |
| feature | 013 | Account Rate-Limit Utilization | [feature/013_account_limits.md](../feature/013_account_limits.md) |
| feature | 014 | Rich Account Metadata | [feature/014_rich_account_metadata.md](../feature/014_rich_account_metadata.md) |
| feature | 015 | Account Name Shortcut Syntax | [feature/015_name_shortcut_syntax.md](../feature/015_name_shortcut_syntax.md) |
| feature | 016 | Current Account Awareness | [feature/016_current_account_awareness.md](../feature/016_current_account_awareness.md) |
| feature | 017 | Expired Token Refresh via Isolated Subprocess | [feature/017_token_refresh.md](../feature/017_token_refresh.md) |
| feature | 018 | Live Quota Monitor Mode | [feature/018_live_monitor.md](../feature/018_live_monitor.md) |
| feature | 019 | Browser Re-Authentication for Named Account | [feature/019_account_relogin.md](../feature/019_account_relogin.md) |
| feature | 020 | Usage Sort Strategies | [feature/020_usage_sort_strategies.md](../feature/020_usage_sort_strategies.md) |
| feature | 021 | Extended Snapshot Fields | [feature/021_extended_snapshot_fields.md](../feature/021_extended_snapshot_fields.md) |
| feature | 022 | Org Identity Snapshot | [feature/022_org_identity_snapshot.md](../feature/022_org_identity_snapshot.md) |
| feature | 023 | Next Account Recommendation Strategies | [feature/023_next_account_strategies.md](../feature/023_next_account_strategies.md) |
| feature | 024 | Session Touch via Isolated Subprocess | [feature/024_session_touch.md](../feature/024_session_touch.md) |
| feature | 025 | Per-Machine Active Marker | [feature/025_per_machine_active_marker.md](../feature/025_per_machine_active_marker.md) |
| feature | 026 | Subprocess Model and Effort Control | [feature/026_subprocess_model_effort.md](../feature/026_subprocess_model_effort.md) |
| feature | 027 | `.account.use` Post-Switch Touch | [feature/027_account_use_post_switch_touch.md](../feature/027_account_use_post_switch_touch.md) |
| feature | 028 | Usage Row Filtering | [feature/028_usage_row_filtering.md](../feature/028_usage_row_filtering.md) |
| feature | 029 | Account Host Metadata | [feature/029_account_host_metadata.md](../feature/029_account_host_metadata.md) |
| feature | 030 | Account Billing Renewal Override | [feature/030_account_renewal_override.md](../feature/030_account_renewal_override.md) |
| feature | 031 | Account Inspect | [feature/031_account_inspect.md](../feature/031_account_inspect.md) |
| feature | 032 | Account Marker Assignment | [feature/032_account_assign.md](../feature/032_account_assign.md) |
| feature | 033 | Quota Cache Fallback | [feature/033_quota_cache.md](../feature/033_quota_cache.md) |
| feature | 034 | Explicit Session Model Override | [feature/034_explicit_session_model_override.md](../feature/034_explicit_session_model_override.md) |
| feature | 035 | Dedicated Model Get/Set Command | [feature/035_model_command.md](../feature/035_model_command.md) |
| feature | 036 | Account Ownership | [feature/036_account_ownership.md](../feature/036_account_ownership.md) |
| feature | 037 | Accounts/Usage Param Unification | [feature/037_accounts_usage_param_unification.md](../feature/037_accounts_usage_param_unification.md) |
| cli | 001 | Config Parameters | [cli/001_config_param.md](../cli/001_config_param.md) |
| cli | 002 | Dictionary | [cli/002_dictionary.md](../cli/002_dictionary.md) |
| cli | 003 | Environment Parameters | [cli/003_env_param.md](../cli/003_env_param.md) |
| cli | 004 | Parameter Interactions | [cli/004_parameter_interactions.md](../cli/004_parameter_interactions.md) |
| command | 001 | Account Commands | [cli/command/001_account.md](../cli/command/001_account.md) |
| command | 002 | Credentials Commands | [cli/command/002_credentials.md](../cli/command/002_credentials.md) |
| command | 003 | Meta Commands | [cli/command/003_meta.md](../cli/command/003_meta.md) |
| command | 004 | Paths Commands | [cli/command/004_paths.md](../cli/command/004_paths.md) |
| command | 005 | Token Commands | [cli/command/005_token.md](../cli/command/005_token.md) |
| command | 006 | Usage Commands | [cli/command/006_usage.md](../cli/command/006_usage.md) |
| command | 007 | Model Command | [cli/command/007_model.md](../cli/command/007_model.md) |
| param | 001 | `name::` | [cli/param/001_name.md](../cli/param/001_name.md) |
| param | 002 | `format::` | [cli/param/002_format.md](../cli/param/002_format.md) |
| param | 003 | `threshold::` | [cli/param/003_threshold.md](../cli/param/003_threshold.md) |
| param | 004 | `dry::` | [cli/param/004_dry.md](../cli/param/004_dry.md) |
| param | 005 | `account::` | [cli/param/005_account.md](../cli/param/005_account.md) |
| param | 006 | `sub::` | [cli/param/006_sub.md](../cli/param/006_sub.md) |
| param | 007 | `tier::` | [cli/param/007_tier.md](../cli/param/007_tier.md) |
| param | 008 | `token::` | [cli/param/008_token.md](../cli/param/008_token.md) |
| param | 009 | `expires::` | [cli/param/009_expires.md](../cli/param/009_expires.md) |
| param | 010 | `email::` | [cli/param/010_email.md](../cli/param/010_email.md) |
| param | 011 | `file::` | [cli/param/011_file.md](../cli/param/011_file.md) |
| param | 012 | `saved::` | [cli/param/012_saved.md](../cli/param/012_saved.md) |
| param | 013 | `active::` | [cli/param/013_active.md](../cli/param/013_active.md) |
| param | 014 | `display_name::` | [cli/param/014_display_name.md](../cli/param/014_display_name.md) |
| param | 015 | `role::` | [cli/param/015_role.md](../cli/param/015_role.md) |
| param | 016 | `billing::` | [cli/param/016_billing.md](../cli/param/016_billing.md) |
| param | 017 | `model::` | [cli/param/017_model.md](../cli/param/017_model.md) |
| param | 018 | `current::` | [cli/param/018_current.md](../cli/param/018_current.md) |
| param | 019 | `refresh::` | [cli/param/019_refresh.md](../cli/param/019_refresh.md) |
| param | 020 | `live::` | [cli/param/020_live.md](../cli/param/020_live.md) |
| param | 021 | `interval::` | [cli/param/021_interval.md](../cli/param/021_interval.md) |
| param | 022 | `jitter::` | [cli/param/022_jitter.md](../cli/param/022_jitter.md) |
| param | 023 | `trace::` | [cli/param/023_trace.md](../cli/param/023_trace.md) |
| param | 024 | `field::` | [cli/param/024_field.md](../cli/param/024_field.md) |
| param | 025 | `sort::` | [cli/param/025_sort.md](../cli/param/025_sort.md) |
| param | 026 | `desc::` | [cli/param/026_desc.md](../cli/param/026_desc.md) |
| param | 027 | `prefer::` | [cli/param/027_prefer.md](../cli/param/027_prefer.md) |
| param | 028 | `uuid::` | [cli/param/028_uuid.md](../cli/param/028_uuid.md) |
| param | 029 | `capabilities::` | [cli/param/029_capabilities.md](../cli/param/029_capabilities.md) |
| param | 030 | `org_uuid::` | [cli/param/030_org_uuid.md](../cli/param/030_org_uuid.md) |
| param | 031 | `org_name::` | [cli/param/031_org_name.md](../cli/param/031_org_name.md) |
| param | 032 | `next::` | [cli/param/032_next.md](../cli/param/032_next.md) |
| param | 033 | `cols::` | [cli/param/033_cols.md](../cli/param/033_cols.md) |
| param | 034 | `touch::` | [cli/param/034_touch.md](../cli/param/034_touch.md) |
| param | 035 | `imodel::` | [cli/param/035_imodel.md](../cli/param/035_imodel.md) |
| param | 036 | `effort::` | [cli/param/036_effort.md](../cli/param/036_effort.md) |
| param | 037 | `count::` | [cli/param/037_count.md](../cli/param/037_count.md) |
| param | 038 | `offset::` | [cli/param/038_offset.md](../cli/param/038_offset.md) |
| param | 039 | `only_active::` | [cli/param/039_only_active.md](../cli/param/039_only_active.md) |
| param | 040 | `only_next::` | [cli/param/040_only_next.md](../cli/param/040_only_next.md) |
| param | 041 | `min_5h::` | [cli/param/041_min_5h.md](../cli/param/041_min_5h.md) |
| param | 042 | `min_7d::` | [cli/param/042_min_7d.md](../cli/param/042_min_7d.md) |
| param | 043 | `only_valid::` | [cli/param/043_only_valid.md](../cli/param/043_only_valid.md) |
| param | 044 | `exclude_exhausted::` | [cli/param/044_exclude_exhausted.md](../cli/param/044_exclude_exhausted.md) |
| param | 045 | `get::` | [cli/param/045_get.md](../cli/param/045_get.md) |
| param | 046 | `abs::` | [cli/param/046_abs.md](../cli/param/046_abs.md) |
| param | 047 | `no_color::` | [cli/param/047_no_color.md](../cli/param/047_no_color.md) |
| param | 048 | `host::` | [cli/param/048_host.md](../cli/param/048_host.md) |
| param | 049 | `at::` | [cli/param/049_at.md](../cli/param/049_at.md) |
| param | 050 | `from_now::` | [cli/param/050_from_now.md](../cli/param/050_from_now.md) |
| param | 051 | `clear::` | [cli/param/051_clear.md](../cli/param/051_clear.md) |
| param | 052 | `role::` (metadata label) | [cli/param/052_role.md](../cli/param/052_role.md) |
| param | 053 | `for::` | [cli/param/053_for.md](../cli/param/053_for.md) |
| param | 054 | `set_model::` | [cli/param/054_set_model.md](../cli/param/054_set_model.md) |
| param | 055 | `set::` | [cli/param/055_set.md](../cli/param/055_set.md) |
| param | 056 | `unclaim::` (re-activated in Feature 037 as mutation param on `.accounts`/`.usage`) | [cli/param/056_unclaim.md](../cli/param/056_unclaim.md) |
| param | 057 | `assign::` (mutation param on `.accounts`/`.usage`, Feature 037) | [cli/param/057_assign.md](../cli/param/057_assign.md) |
| param | 058 | `force::` (bypass G5–G8 ownership enforcement) | [cli/param/058_force.md](../cli/param/058_force.md) |
| param_group | 001 | Output Control | [cli/param_group/001_output_control.md](../cli/param_group/001_output_control.md) |
| param_group | 002 | Field Presence | [cli/param_group/002_field_presence.md](../cli/param_group/002_field_presence.md) |
| param_group | 003 | Fetch Behavior | [cli/param_group/003_fetch_behavior.md](../cli/param_group/003_fetch_behavior.md) |
| param_group | 004 | Sort Control | [cli/param_group/004_sort_control.md](../cli/param_group/004_sort_control.md) |
| param_group | 005 | Display Control | [cli/param_group/005_display_control.md](../cli/param_group/005_display_control.md) |
| param_group | 006 | Account Targeting | [cli/param_group/006_account_targeting.md](../cli/param_group/006_account_targeting.md) |
| type | 001 | AccountName | [cli/type/001_account_name.md](../cli/type/001_account_name.md) |
| type | 002 | OutputFormat | [cli/type/002_output_format.md](../cli/type/002_output_format.md) |
| type | 003 | WarningThreshold | [cli/type/003_warning_threshold.md](../cli/type/003_warning_threshold.md) |
| type | 004 | AccountSelector | [cli/type/004_account_selector.md](../cli/type/004_account_selector.md) |
| format | 001 | text | [cli/format/001_text.md](../cli/format/001_text.md) |
| format | 002 | json | [cli/format/002_json.md](../cli/format/002_json.md) |
| format | 003 | table | [cli/format/003_table.md](../cli/format/003_table.md) |
| user_story | 001 | Account Rotation | [cli/user_story/001_account_rotation.md](../cli/user_story/001_account_rotation.md) |
| user_story | 002 | Account Onboarding | [cli/user_story/002_onboarding.md](../cli/user_story/002_onboarding.md) |
| user_story | 003 | Quota Monitoring | [cli/user_story/003_quota_monitoring.md](../cli/user_story/003_quota_monitoring.md) |
| user_story | 004 | Scripted Automation | [cli/user_story/004_scripted_automation.md](../cli/user_story/004_scripted_automation.md) |
| user_story | 005 | Credential Diagnostics | [cli/user_story/005_credential_diagnostics.md](../cli/user_story/005_credential_diagnostics.md) |
| command_noun | 001 | Account Noun | [cli/command_noun/001_account.md](../cli/command_noun/001_account.md) |
| command_noun | 002 | Token Noun | [cli/command_noun/002_token.md](../cli/command_noun/002_token.md) |
| command_noun | 003 | Credentials Noun | [cli/command_noun/003_credentials.md](../cli/command_noun/003_credentials.md) |
| command_verb | 001 | save | [cli/command_verb/001_save.md](../cli/command_verb/001_save.md) |
| command_verb | 002 | use | [cli/command_verb/002_use.md](../cli/command_verb/002_use.md) |
| command_verb | 003 | delete | [cli/command_verb/003_delete.md](../cli/command_verb/003_delete.md) |
| command_verb | 004 | limits | [cli/command_verb/004_limits.md](../cli/command_verb/004_limits.md) |
| command_verb | 005 | relogin | [cli/command_verb/005_relogin.md](../cli/command_verb/005_relogin.md) |
| command_verb | 006 | rotate | [cli/command_verb/006_rotate.md](../cli/command_verb/006_rotate.md) |
| command_verb | 007 | renewal | [cli/command_verb/007_renewal.md](../cli/command_verb/007_renewal.md) |
| command_verb | 008 | inspect | [cli/command_verb/008_inspect.md](../cli/command_verb/008_inspect.md) |
| command_verb | 009 | assign | [cli/command_verb/009_assign.md](../cli/command_verb/009_assign.md) |
| command_verb | 010 | status | [cli/command_verb/010_status.md](../cli/command_verb/010_status.md) |
| command_verb | 011 | unclaim | [cli/command_verb/011_unclaim.md](../cli/command_verb/011_unclaim.md) |
| invariant | 001 | Zero Third-Party Dependencies | [invariant/001_zero_third_party_deps.md](../invariant/001_zero_third_party_deps.md) |
| invariant | 002 | Cross-Platform Compatibility | [invariant/002_cross_platform.md](../invariant/002_cross_platform.md) |
| invariant | 003 | Clear Error Messages | [invariant/003_clear_errors.md](../invariant/003_clear_errors.md) |
| invariant | 004 | No Process Execution | [invariant/004_no_process_execution.md](../invariant/004_no_process_execution.md) |
| invariant | 005 | Atomic Account Switching | [invariant/005_atomic_switching.md](../invariant/005_atomic_switching.md) |
| invariant | 006 | Parameters Default to Active Context | [invariant/006_param_defaults.md](../invariant/006_param_defaults.md) |
| research_interactive | 001 | Claude Interactive Session Control | [research_interactive/001_claude_interactive_session_control.md](../research_interactive/001_claude_interactive_session_control.md) |
