# Doc Entities

## Master Doc Entities Table

| Type | Purpose | Master File | Instances |
|------|---------|-------------|----------:|
| `behavior` | Observed external behaviors of the `claude` binary (B1–B37 + B16h) | [behavior/readme.md](behavior/readme.md) | 38 |
| `storage` | `~/.claude/` storage architecture: projects dir, support dirs, root files | [storage/readme.md](storage/readme.md) | 3 |
| `filesystem` | Runtime filesystem paths accessed by claude_version | [filesystem/readme.md](filesystem/readme.md) | 4 |
| `jsonl` | Session JSONL entry format: common fields, entry types, content blocks, usage, threading, sidechain | [jsonl/readme.md](jsonl/readme.md) | 10 |
| `envelope` | Session-log top-level line kinds: one instance per `type` discriminator, with payload fields, structural class, and version lifecycle | [envelope/readme.md](envelope/readme.md) | 19 |
| `attachment` | Harness context-injection payloads: one instance per `attachment.type`, the second dispatch level | [attachment/readme.md](attachment/readme.md) | 23 |
| `system_event` | Session lifecycle, telemetry, and error events: one instance per `system.subtype`, the third dispatch level | [system_event/readme.md](system_event/readme.md) | 10 |
| `envelope_class` | Common-field presence contract: three classes partitioning all 19 top-level kinds by which common fields they guarantee | [envelope_class/readme.md](envelope_class/readme.md) | 3 |
| `settings` | Settings file structure and protocols: global, project, version lock | [settings/readme.md](settings/readme.md) | 3 |
| `formats` | Data formats: file formats, output schemas — history, credentials, debug, shell-snapshots, tasks, commands, JSON response | [format/readme.md](format/readme.md) | 7 |
| `taxonomy` | Four-level concept hierarchy: Project, Conversation, Session, Entry | [taxonomy/readme.md](taxonomy/readme.md) | 3 |
| `params` | CLI parameter specifications for the `claude` binary | [param/readme.md](param/readme.md) | 159 |
| `tool` | Built-in tools available in Claude Code sessions | [tool/readme.md](tool/readme.md) | 43 |
| `endpoint` | Wire contracts for Anthropic HTTP endpoints consumed by workspace crates | [endpoint/readme.md](endpoint/readme.md) | 11 + 1 index |
| `subcommand` | CLI subcommands of the `claude` binary — 12 listed in `claude --help`, 7 functional but hidden from it | [subcommand/readme.md](subcommand/readme.md) | 19 |
| `fault` | Aggregated index of all fault conditions: terminal errors (E1–E6), silent failures (F1–F4), and quirks (Q1–Q5) with detection signals and `classify_error()` priority order | [fault/readme.md](fault/readme.md) | 0 (index-only) |
| `model` | Claude API model catalog: known model IDs, capabilities, and workspace selection defaults | [model/readme.md](model/readme.md) | 13 |
| `version` | Claude Code release changelog: one doc instance per published release version | [version/readme.md](version/readme.md) | 116 |
| `pattern` | Reusable design-pattern documentation: official version-pinning landscape | [pattern/readme.md](pattern/readme.md) | 1 |

**Total doc instances**: 485, as of 2026-08-27 (38 behavior + 3 storage + 4 filesystem + 10 jsonl + 19 envelope + 23 attachment + 10 system_event + 3 envelope_class + 3 settings + 7 format + 3 taxonomy + 159 param + 43 tool + 11 endpoint + 19 subcommand + 0 fault + 13 model + 116 version + 1 pattern). Plus one unnumbered auxiliary index, `endpoint/account_field_index.md`, which is a cross-endpoint field dictionary rather than an instance and is excluded from the count.

One collection holds no numbered instances: `fault` is **index-only by design** — its readme is itself the complete fault table, and its rows link out to `docs/error/` in the workspace root rather than to local instances; a zero here is correct, not a gap.

Counts in this table are verified against the filesystem with:

```bash
cd contract/claude_code/docs
for d in */; do printf '%-12s %3d\n' "${d%/}" "$(ls "$d" | grep -cE '^[0-9]{3}_')"; done
```

That loop counts strictly-numbered files and totals **484**; `behavior` reports
37 rather than the 38 above, because `016h_b16h_tools_system_prompt.md` carries a
suffixed number. Adding it back gives the 485 stated. `grep -P` is not available
on every platform, which is why the expression above uses POSIX classes.

## Master Doc Instances Table

### behavior/ (38 instances)

| ID | Name | File |
|----|------|------|
| B1 | Default New Session | [behavior/001_b1_default_new_session.md](behavior/001_b1_default_new_session.md) |
| B2 | New Session Creates File | [behavior/002_b2_new_session_creates_file.md](behavior/002_b2_new_session_creates_file.md) |
| B3 | Print Flag Orthogonal | [behavior/003_b3_print_orthogonal.md](behavior/003_b3_print_orthogonal.md) |
| B4 | Continue Flag | [behavior/004_b4_continue_flag.md](behavior/004_b4_continue_flag.md) |
| B5 | Continue Session Selection Rule | [behavior/005_b5_mtime_selection.md](behavior/005_b5_mtime_selection.md) |
| B6 | Session Accumulation | [behavior/006_b6_session_accumulation.md](behavior/006_b6_session_accumulation.md) |
| B7 | Agent Sessions Sibling | [behavior/007_b7_agent_sessions_sibling.md](behavior/007_b7_agent_sessions_sibling.md) |
| B8 | Zero-Byte Placeholder | [behavior/008_b8_zero_byte_placeholder.md](behavior/008_b8_zero_byte_placeholder.md) |
| B9 | Storage Path Encoding | [behavior/009_b9_storage_path_encoding.md](behavior/009_b9_storage_path_encoding.md) |
| B10 | Entry Threading | [behavior/010_b10_entry_threading.md](behavior/010_b10_entry_threading.md) |
| B11 | Auto Continue Env — ❌ refuted | [behavior/011_b11_auto_continue_env.md](behavior/011_b11_auto_continue_env.md) |
| B12 | Agent Session ID | [behavior/012_b12_agent_session_id.md](behavior/012_b12_agent_session_id.md) |
| B13 | Subagent Directory | [behavior/013_b13_subagent_directory.md](behavior/013_b13_subagent_directory.md) |
| B14 | Agent Meta JSON | [behavior/014_b14_agent_meta_json.md](behavior/014_b14_agent_meta_json.md) |
| B15 | Agent Slug | [behavior/015_b15_agent_slug.md](behavior/015_b15_agent_slug.md) |
| B16 | Tools Flag | [behavior/016_b16_tools_flag.md](behavior/016_b16_tools_flag.md) |
| B16h | Tools System Prompt | [behavior/016h_b16h_tools_system_prompt.md](behavior/016h_b16h_tools_system_prompt.md) |
| B17 | parentUuid Self-Contained | [behavior/017_b17_parentuuid_self_contained.md](behavior/017_b17_parentuuid_self_contained.md) |
| B18 | No Cross-Session Links | [behavior/018_b18_no_cross_session_links.md](behavior/018_b18_no_cross_session_links.md) |
| B19 | Resume Flag | [behavior/019_b19_resume_flag.md](behavior/019_b19_resume_flag.md) |
| B20 | Session-ID Flag | [behavior/020_b20_session_id_flag.md](behavior/020_b20_session_id_flag.md) |
| B21 | Fork Session | [behavior/021_b21_fork_session.md](behavior/021_b21_fork_session.md) |
| B22 | No Session Persistence | [behavior/022_b22_no_session_persistence.md](behavior/022_b22_no_session_persistence.md) |
| B23 | Session Dir Override — ❌ refuted | [behavior/023_b23_session_dir_override.md](behavior/023_b23_session_dir_override.md) |
| B24 | From PR | [behavior/024_b24_from_pr.md](behavior/024_b24_from_pr.md) |
| B25 | Auto Compact Window | [behavior/025_b25_auto_compact_window.md](behavior/025_b25_auto_compact_window.md) |
| B26 | Autocompact Pct Override | [behavior/026_b26_autocompact_pct_override.md](behavior/026_b26_autocompact_pct_override.md) |
| B27 | Agent No OS Process | [behavior/027_b27_agent_no_os_process.md](behavior/027_b27_agent_no_os_process.md) |
| B28 | Bash rtk Subprocess | [behavior/028_b28_bash_rtk_subprocess.md](behavior/028_b28_bash_rtk_subprocess.md) |
| B29 | Bash CLAUDE_* Env | [behavior/029_b29_bash_claude_env.md](behavior/029_b29_bash_claude_env.md) |
| B30 | Subagent Context Inheritance | [behavior/030_b30_subagent_context_inheritance.md](behavior/030_b30_subagent_context_inheritance.md) |
| B31 | Subagent Tool Sets | [behavior/031_b31_subagent_tool_sets.md](behavior/031_b31_subagent_tool_sets.md) |
| B32 | claudemd At-Ref Path Filter | [behavior/032_b32_claudemd_at_ref_path_filter.md](behavior/032_b32_claudemd_at_ref_path_filter.md) |
| B33 | claudemd Loading Limits | [behavior/033_b33_claudemd_loading_limits.md](behavior/033_b33_claudemd_loading_limits.md) |
| B34 | claudemd Content Pipeline | [behavior/034_b34_claudemd_content_pipeline.md](behavior/034_b34_claudemd_content_pipeline.md) |
| B35 | Automemory Search Context Flag | [behavior/035_b35_automemory_search_context_flag.md](behavior/035_b35_automemory_search_context_flag.md) |
| B36 | Background Task Lifecycle | [behavior/036_b36_background_task_lifecycle.md](behavior/036_b36_background_task_lifecycle.md) |
| B37 | Subagent Cache Isolation and 5-Minute TTL | [behavior/037_b37_subagent_cache_ttl.md](behavior/037_b37_subagent_cache_ttl.md) |

### storage/ (3 instances)

| ID | Name | File |
|----|------|------|
| 001 | Projects Directory | [storage/001_projects_directory.md](storage/001_projects_directory.md) |
| 002 | Support Directories | [storage/002_support_directories.md](storage/002_support_directories.md) |
| 003 | Root Files | [storage/003_root_files.md](storage/003_root_files.md) |

### filesystem/ (4 instances)

| ID | Name | File |
|----|------|------|
| 001 | Claude Home | [filesystem/001_claude_home.md](filesystem/001_claude_home.md) |
| 002 | Local Install | [filesystem/002_local_install.md](filesystem/002_local_install.md) |
| 003 | Credential Store | [filesystem/003_credential_store.md](filesystem/003_credential_store.md) |
| 004 | Proc System | [filesystem/004_proc_system.md](filesystem/004_proc_system.md) |

### jsonl/ (10 instances)

| ID | Name | File |
|----|------|------|
| 001 | Common Fields | [jsonl/001_common_fields.md](jsonl/001_common_fields.md) |
| 002 | User Entry | [jsonl/002_user_entry.md](jsonl/002_user_entry.md) |
| 003 | Assistant Entry | [jsonl/003_assistant_entry.md](jsonl/003_assistant_entry.md) |
| 004 | Text Block | [jsonl/004_text_block.md](jsonl/004_text_block.md) |
| 005 | Thinking Block | [jsonl/005_thinking_block.md](jsonl/005_thinking_block.md) |
| 006 | Tool Use Block | [jsonl/006_tool_use_block.md](jsonl/006_tool_use_block.md) |
| 007 | Tool Result Block | [jsonl/007_tool_result_block.md](jsonl/007_tool_result_block.md) |
| 008 | Usage Object | [jsonl/008_usage_object.md](jsonl/008_usage_object.md) |
| 009 | Threading Model | [jsonl/009_threading_model.md](jsonl/009_threading_model.md) |
| 010 | Sidechain Sessions | [jsonl/010_sidechain_sessions.md](jsonl/010_sidechain_sessions.md) |

### envelope/ (19 instances)

| ID | Name | File |
|----|------|------|
| 001 | Assistant | [envelope/001_assistant.md](envelope/001_assistant.md) |
| 002 | User | [envelope/002_user.md](envelope/002_user.md) |
| 003 | Attachment | [envelope/003_attachment.md](envelope/003_attachment.md) |
| 004 | Last Prompt | [envelope/004_last_prompt.md](envelope/004_last_prompt.md) |
| 005 | Mode | [envelope/005_mode.md](envelope/005_mode.md) |
| 006 | AI Title | [envelope/006_ai_title.md](envelope/006_ai_title.md) |
| 007 | Permission Mode | [envelope/007_permission_mode.md](envelope/007_permission_mode.md) |
| 008 | Queue Operation | [envelope/008_queue_operation.md](envelope/008_queue_operation.md) |
| 009 | System | [envelope/009_system.md](envelope/009_system.md) |
| 010 | Progress | [envelope/010_progress.md](envelope/010_progress.md) |
| 011 | Agent Name | [envelope/011_agent_name.md](envelope/011_agent_name.md) |
| 012 | File History Snapshot | [envelope/012_file_history_snapshot.md](envelope/012_file_history_snapshot.md) |
| 013 | Custom Title | [envelope/013_custom_title.md](envelope/013_custom_title.md) |
| 014 | PR Link | [envelope/014_pr_link.md](envelope/014_pr_link.md) |
| 015 | Started | [envelope/015_started.md](envelope/015_started.md) |
| 016 | Result | [envelope/016_result.md](envelope/016_result.md) |
| 017 | Summary | [envelope/017_summary.md](envelope/017_summary.md) |
| 018 | Fork Context Ref | [envelope/018_fork_context_ref.md](envelope/018_fork_context_ref.md) |
| 019 | Frame Link | [envelope/019_frame_link.md](envelope/019_frame_link.md) |

### attachment/ (23 instances)

| ID | Name | File |
|----|------|------|
| 001 | Total Tokens Reminder | [attachment/001_total_tokens_reminder.md](attachment/001_total_tokens_reminder.md) |
| 002 | Task Reminder | [attachment/002_task_reminder.md](attachment/002_task_reminder.md) |
| 003 | Compact File Reference | [attachment/003_compact_file_reference.md](attachment/003_compact_file_reference.md) |
| 004 | Deferred Tools Delta | [attachment/004_deferred_tools_delta.md](attachment/004_deferred_tools_delta.md) |
| 005 | File | [attachment/005_file.md](attachment/005_file.md) |
| 006 | Skill Listing | [attachment/006_skill_listing.md](attachment/006_skill_listing.md) |
| 007 | Agent Listing Delta | [attachment/007_agent_listing_delta.md](attachment/007_agent_listing_delta.md) |
| 008 | Invoked Skills | [attachment/008_invoked_skills.md](attachment/008_invoked_skills.md) |
| 009 | Ultrathink Effort | [attachment/009_ultrathink_effort.md](attachment/009_ultrathink_effort.md) |
| 010 | Queued Command | [attachment/010_queued_command.md](attachment/010_queued_command.md) |
| 011 | Command Permissions | [attachment/011_command_permissions.md](attachment/011_command_permissions.md) |
| 012 | MCP Instructions Delta | [attachment/012_mcp_instructions_delta.md](attachment/012_mcp_instructions_delta.md) |
| 013 | Date Change | [attachment/013_date_change.md](attachment/013_date_change.md) |
| 014 | Task Status | [attachment/014_task_status.md](attachment/014_task_status.md) |
| 015 | Read Truncation Notice | [attachment/015_read_truncation_notice.md](attachment/015_read_truncation_notice.md) |
| 016 | Edited Text File | [attachment/016_edited_text_file.md](attachment/016_edited_text_file.md) |
| 017 | Plan File Reference | [attachment/017_plan_file_reference.md](attachment/017_plan_file_reference.md) |
| 018 | Plan Mode | [attachment/018_plan_mode.md](attachment/018_plan_mode.md) |
| 019 | Nested Memory | [attachment/019_nested_memory.md](attachment/019_nested_memory.md) |
| 020 | Plan Mode Exit | [attachment/020_plan_mode_exit.md](attachment/020_plan_mode_exit.md) |
| 021 | Plan Mode Reentry | [attachment/021_plan_mode_reentry.md](attachment/021_plan_mode_reentry.md) |
| 022 | Hook Additional Context | [attachment/022_hook_additional_context.md](attachment/022_hook_additional_context.md) |
| 023 | Context Tip | [attachment/023_context_tip.md](attachment/023_context_tip.md) |

### system_event/ (10 instances)

| ID | Name | File |
|----|------|------|
| 001 | Compact Boundary | [system_event/001_compact_boundary.md](system_event/001_compact_boundary.md) |
| 002 | Local Command | [system_event/002_local_command.md](system_event/002_local_command.md) |
| 003 | Turn Duration | [system_event/003_turn_duration.md](system_event/003_turn_duration.md) |
| 004 | Away Summary | [system_event/004_away_summary.md](system_event/004_away_summary.md) |
| 005 | API Error | [system_event/005_api_error.md](system_event/005_api_error.md) |
| 006 | Bridge Status | [system_event/006_bridge_status.md](system_event/006_bridge_status.md) |
| 007 | Model Consent Fallback | [system_event/007_model_consent_fallback.md](system_event/007_model_consent_fallback.md) |
| 008 | Scheduled Task Fire | [system_event/008_scheduled_task_fire.md](system_event/008_scheduled_task_fire.md) |
| 009 | Informational | [system_event/009_informational.md](system_event/009_informational.md) |
| 010 | Agents Killed | [system_event/010_agents_killed.md](system_event/010_agents_killed.md) |

### envelope_class/ (3 instances)

| ID | Name | File |
|----|------|------|
| 001 | Full Envelope | [envelope_class/001_full_envelope.md](envelope_class/001_full_envelope.md) |
| 002 | Session-Scoped | [envelope_class/002_session_scoped.md](envelope_class/002_session_scoped.md) |
| 003 | Detached | [envelope_class/003_detached.md](envelope_class/003_detached.md) |

### settings/ (3 instances)

| ID | Name | File |
|----|------|------|
| 001 | Global Settings | [settings/001_global_settings.md](settings/001_global_settings.md) |
| 002 | Project Settings | [settings/002_project_settings.md](settings/002_project_settings.md) |
| 003 | Version Lock | [settings/003_version_lock.md](settings/003_version_lock.md) |

### format/ (7 instances)

| ID | Name | File |
|----|------|------|
| 001 | History JSONL | [format/001_history_jsonl.md](format/001_history_jsonl.md) |
| 002 | Credentials | [format/002_credentials.md](format/002_credentials.md) |
| 003 | Debug Log | [format/003_debug_log.md](format/003_debug_log.md) |
| 004 | Shell Snapshot | [format/004_shell_snapshot.md](format/004_shell_snapshot.md) |
| 005 | Task | [format/005_task.md](format/005_task.md) |
| 006 | Command Definition | [format/006_command_definition.md](format/006_command_definition.md) |
| 007 | JSON Response | [format/007_json_response.md](format/007_json_response.md) |

### taxonomy/ (3 instances)

| ID | Name | File |
|----|------|------|
| 001 | Concepts | [taxonomy/001_concepts.md](taxonomy/001_concepts.md) |
| 002 | Relationships | [taxonomy/002_relationships.md](taxonomy/002_relationships.md) |
| 003 | Implementation | [taxonomy/003_implementation.md](taxonomy/003_implementation.md) |

### endpoint/ (11 instances)

| ID | Name | File |
|----|------|------|
| 001 | OAuth Usage | [endpoint/001_oauth_usage.md](endpoint/001_oauth_usage.md) |
| 002 | OAuth Account | [endpoint/002_oauth_account.md](endpoint/002_oauth_account.md) |
| 003 | Messages Rate-Limit Headers | [endpoint/003_v1_messages.md](endpoint/003_v1_messages.md) |
| 004 | OAuth Token Refresh | [endpoint/004_oauth_token.md](endpoint/004_oauth_token.md) |
| 005 | Claude CLI Roles | [endpoint/005_claude_cli_roles.md](endpoint/005_claude_cli_roles.md) |
| 006 | Create API Key | [endpoint/006_create_api_key.md](endpoint/006_create_api_key.md) |
| 007 | Metrics Enabled | [endpoint/007_metrics_enabled.md](endpoint/007_metrics_enabled.md) |
| 008 | Shared Session Transcripts | [endpoint/008_shared_session_transcripts.md](endpoint/008_shared_session_transcripts.md) |
| 009 | CLI Feedback | [endpoint/009_cli_feedback.md](endpoint/009_cli_feedback.md) |
| 010 | Web Domain Info | [endpoint/010_web_domain_info.md](endpoint/010_web_domain_info.md) |
| 011 | List Models | [endpoint/011_v1_models.md](endpoint/011_v1_models.md) |
| — | Account Field Index (auxiliary, unnumbered) | [endpoint/account_field_index.md](endpoint/account_field_index.md) |

### subcommand/ (19 instances)

| ID | Name | File |
|----|------|------|
| 001 | agents | [subcommand/001_agents.md](subcommand/001_agents.md) |
| 002 | auth | [subcommand/002_auth.md](subcommand/002_auth.md) |
| 003 | auto-mode | [subcommand/003_auto_mode.md](subcommand/003_auto_mode.md) |
| 004 | doctor | [subcommand/004_doctor.md](subcommand/004_doctor.md) |
| 005 | install | [subcommand/005_install.md](subcommand/005_install.md) |
| 006 | mcp | [subcommand/006_mcp.md](subcommand/006_mcp.md) |
| 007 | plugin | [subcommand/007_plugin.md](subcommand/007_plugin.md) |
| 008 | setup-token | [subcommand/008_setup_token.md](subcommand/008_setup_token.md) |
| 009 | update | [subcommand/009_update.md](subcommand/009_update.md) |
| 010 | gateway | [subcommand/010_gateway.md](subcommand/010_gateway.md) |
| 011 | project | [subcommand/011_project.md](subcommand/011_project.md) |
| 012 | ultrareview | [subcommand/012_ultrareview.md](subcommand/012_ultrareview.md) |
| 013 | attach *(hidden)* | [subcommand/013_attach.md](subcommand/013_attach.md) |
| 014 | daemon *(hidden)* | [subcommand/014_daemon.md](subcommand/014_daemon.md) |
| 015 | import *(hidden)* | [subcommand/015_import.md](subcommand/015_import.md) |
| 016 | logs *(hidden)* | [subcommand/016_logs.md](subcommand/016_logs.md) |
| 017 | respawn *(hidden)* | [subcommand/017_respawn.md](subcommand/017_respawn.md) |
| 018 | rm *(hidden)* | [subcommand/018_rm.md](subcommand/018_rm.md) |
| 019 | stop *(hidden)* | [subcommand/019_stop.md](subcommand/019_stop.md) |

> `params` instances (159 files) use numbered naming and are enumerated in their master file: [param/readme.md](param/readme.md). Numbers 001–140 are alphabetical by parameter name; 141–159 append later additions, since renumbering would break every cross-reference into the collection.
>
> `tool` instances (43 files) use numbered naming and are enumerated in their master file: [tool/readme.md](tool/readme.md). Numbers 041–043 (`DesignSync`, `EndConversation`, `ReportFindings`) were found by inspecting a live v2.1.220 session's tool listing — none appears in `--help` or in any release note.
>
> `subcommand` instances (19 files) use numbered naming and are enumerated in their master file: [subcommand/readme.md](subcommand/readme.md).
>
> `version` instances (116 files) use NNN_vX_Y_Z.md naming and are enumerated in their master file: [version/readme.md](version/readme.md).

### model/ (13 instances)

| ID | Name | File |
|----|------|------|
| 001 | claude-fable-5 | [model/001_claude_fable_5.md](model/001_claude_fable_5.md) |
| 002 | claude-mythos-5 | [model/002_claude_mythos_5.md](model/002_claude_mythos_5.md) |
| 003 | claude-opus-4-8 | [model/003_claude_opus_4_8.md](model/003_claude_opus_4_8.md) |
| 004 | claude-sonnet-5 | [model/004_claude_sonnet_5.md](model/004_claude_sonnet_5.md) |
| 005 | claude-haiku-4-5 | [model/005_claude_haiku_4_5.md](model/005_claude_haiku_4_5.md) |
| 006 | claude-opus-4-7 | [model/006_claude_opus_4_7.md](model/006_claude_opus_4_7.md) |
| 007 | claude-opus-4-6 | [model/007_claude_opus_4_6.md](model/007_claude_opus_4_6.md) |
| 008 | claude-sonnet-4-6 | [model/008_claude_sonnet_4_6.md](model/008_claude_sonnet_4_6.md) |
| 009 | claude-sonnet-4-5 | [model/009_claude_sonnet_4_5.md](model/009_claude_sonnet_4_5.md) |
| 010 | claude-opus-4-5 | [model/010_claude_opus_4_5.md](model/010_claude_opus_4_5.md) |
| 011 | claude-opus-4-1 | [model/011_claude_opus_4_1.md](model/011_claude_opus_4_1.md) |
| 012 | Workspace Defaults | [model/012_workspace_defaults.md](model/012_workspace_defaults.md) |
| 013 | claude-opus-5 *(current default Opus)* | [model/013_claude_opus_5.md](model/013_claude_opus_5.md) |

### pattern/ (1 instance)

| ID | Name | File |
|----|------|------|
| 001 | Version Pinning | [pattern/001_version_pinning.md](pattern/001_version_pinning.md) |
