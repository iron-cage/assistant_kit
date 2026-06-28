# Doc Entities

## Master Doc Entities Table

| Type | Purpose | Master File | Instances |
|------|---------|-------------|----------:|
| `behavior` | Observed external behaviors of the `claude` binary (B1–B31 + B16h) | [behavior/readme.md](behavior/readme.md) | 32 |
| `storage` | `~/.claude/` storage architecture: projects dir, support dirs, root files | [storage/readme.md](storage/readme.md) | 3 |
| `filesystem` | Runtime filesystem paths accessed by claude_version | [filesystem/readme.md](filesystem/readme.md) | 4 |
| `jsonl` | Session JSONL entry format: common fields, entry types, content blocks, usage, threading, sidechain | [jsonl/readme.md](jsonl/readme.md) | 10 |
| `settings` | Settings file structure and protocols: global, project, version lock | [settings/readme.md](settings/readme.md) | 3 |
| `formats` | Data formats: file formats, output schemas — history, credentials, debug, shell-snapshots, todos, commands, JSON response | [format/readme.md](format/readme.md) | 7 |
| `taxonomy` | Four-level concept hierarchy: Project, Conversation, Session, Entry | [taxonomy/readme.md](taxonomy/readme.md) | 3 |
| `params` | CLI parameter specifications for the `claude` binary | [param/readme.md](param/readme.md) | 120 |
| `tool` | Built-in tools available in Claude Code sessions | [tool/readme.md](tool/readme.md) | 40 |
| `endpoint` | Wire contracts for Anthropic HTTP endpoints consumed by workspace crates | [endpoint/readme.md](endpoint/readme.md) | 10 |
| `subcommand` | CLI subcommands: agents, auth, auto-mode, doctor, install, mcp, plugin, setup-token, update | [subcommand/readme.md](subcommand/readme.md) | 9 |
| `fault` | Aggregated index of all fault conditions: terminal errors (E1–E6), silent failures (F1–F4), and quirks (Q1–Q5) with detection signals and `classify_error()` priority order | [fault/readme.md](fault/readme.md) | 0 |

**Total doc instances**: 241 (32 + 3 + 4 + 10 + 3 + 7 + 3 + 120 + 40 + 10 + 9)

## Master Doc Instances Table

### behavior/ (32 instances)

| ID | Name | File |
|----|------|------|
| B1 | Default New Session | [behavior/001_b1_default_new_session.md](behavior/001_b1_default_new_session.md) |
| B2 | New Session Creates File | [behavior/002_b2_new_session_creates_file.md](behavior/002_b2_new_session_creates_file.md) |
| B3 | Print Flag Orthogonal | [behavior/003_b3_print_orthogonal.md](behavior/003_b3_print_orthogonal.md) |
| B4 | Continue Flag | [behavior/004_b4_continue_flag.md](behavior/004_b4_continue_flag.md) |
| B5 | Mtime Selection | [behavior/005_b5_mtime_selection.md](behavior/005_b5_mtime_selection.md) |
| B6 | Session Accumulation | [behavior/006_b6_session_accumulation.md](behavior/006_b6_session_accumulation.md) |
| B7 | Agent Sessions Sibling | [behavior/007_b7_agent_sessions_sibling.md](behavior/007_b7_agent_sessions_sibling.md) |
| B8 | Zero-Byte Placeholder | [behavior/008_b8_zero_byte_placeholder.md](behavior/008_b8_zero_byte_placeholder.md) |
| B9 | Storage Path Encoding | [behavior/009_b9_storage_path_encoding.md](behavior/009_b9_storage_path_encoding.md) |
| B10 | Entry Threading | [behavior/010_b10_entry_threading.md](behavior/010_b10_entry_threading.md) |
| B11 | Auto Continue Env | [behavior/011_b11_auto_continue_env.md](behavior/011_b11_auto_continue_env.md) |
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
| B23 | Session Dir Override | [behavior/023_b23_session_dir_override.md](behavior/023_b23_session_dir_override.md) |
| B24 | From PR | [behavior/024_b24_from_pr.md](behavior/024_b24_from_pr.md) |
| B25 | Auto Compact Window | [behavior/025_b25_auto_compact_window.md](behavior/025_b25_auto_compact_window.md) |
| B26 | Autocompact Pct Override | [behavior/026_b26_autocompact_pct_override.md](behavior/026_b26_autocompact_pct_override.md) |
| B27 | Agent No OS Process | [behavior/027_b27_agent_no_os_process.md](behavior/027_b27_agent_no_os_process.md) |
| B28 | Bash rtk Subprocess | [behavior/028_b28_bash_rtk_subprocess.md](behavior/028_b28_bash_rtk_subprocess.md) |
| B29 | Bash CLAUDE_* Env | [behavior/029_b29_bash_claude_env.md](behavior/029_b29_bash_claude_env.md) |
| B30 | Subagent Context Inheritance | [behavior/030_b30_subagent_context_inheritance.md](behavior/030_b30_subagent_context_inheritance.md) |
| B31 | Subagent Tool Sets | [behavior/031_b31_subagent_tool_sets.md](behavior/031_b31_subagent_tool_sets.md) |

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
| 005 | Todo | [format/005_todo.md](format/005_todo.md) |
| 006 | Command Definition | [format/006_command_definition.md](format/006_command_definition.md) |
| 007 | JSON Response | [format/007_json_response.md](format/007_json_response.md) |

### taxonomy/ (3 instances)

| ID | Name | File |
|----|------|------|
| 001 | Concepts | [taxonomy/001_concepts.md](taxonomy/001_concepts.md) |
| 002 | Relationships | [taxonomy/002_relationships.md](taxonomy/002_relationships.md) |
| 003 | Implementation | [taxonomy/003_implementation.md](taxonomy/003_implementation.md) |

### endpoint/ (10 instances)

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

### subcommand/ (9 instances)

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

> `params` instances (120 files) use numbered naming and are enumerated in their master file: [param/readme.md](param/readme.md).
>
> `tool` instances (40 files) use numbered naming and are enumerated in their master file: [tool/readme.md](tool/readme.md).
>
> `subcommand` instances (9 files) use numbered naming and are enumerated in their master file: [subcommand/readme.md](subcommand/readme.md).
