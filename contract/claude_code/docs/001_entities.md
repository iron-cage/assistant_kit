# Doc Entities

## Master Doc Entities Table

| Type | Purpose | Master File | Instances |
|------|---------|-------------|----------:|
| `behavior` | Behavioral concept specifications for the `claude` binary | [behavior/readme.md](behavior/readme.md) | 7 |
| `params` | CLI parameter specifications for the `claude` binary | [params/readme.md](params/readme.md) | 73 |
| `endpoint` | Wire contracts for Anthropic HTTP endpoints consumed by workspace crates | [endpoint/readme.md](endpoint/readme.md) | 10 |

## Master Doc Instances Table

| Entity | ID | Name | File |
|--------|----|------|------|
| behavior | 001 | Session Behaviors | [behavior/001_session_behaviors.md](behavior/001_session_behaviors.md) |
| behavior | 002 | Storage Organization | [behavior/002_storage_organization.md](behavior/002_storage_organization.md) |
| behavior | 003 | Filesystem Layout | [behavior/003_filesystem_layout.md](behavior/003_filesystem_layout.md) |
| behavior | 004 | JSONL Format | [behavior/004_jsonl_format.md](behavior/004_jsonl_format.md) |
| behavior | 005 | Settings Format | [behavior/005_settings_format.md](behavior/005_settings_format.md) |
| behavior | 006 | Ancillary Formats | [behavior/006_ancillary_formats.md](behavior/006_ancillary_formats.md) |
| behavior | 007 | Concept Taxonomy | [behavior/007_concept_taxonomy.md](behavior/007_concept_taxonomy.md) |
| endpoint | 001 | OAuth Usage | [endpoint/001_oauth_usage.md](endpoint/001_oauth_usage.md) |
| endpoint | 002 | OAuth Account | [endpoint/002_oauth_account.md](endpoint/002_oauth_account.md) |
| endpoint | 003 | Messages Rate-Limit Headers | [endpoint/003_v1_messages.md](endpoint/003_v1_messages.md) |
| endpoint | 004 | OAuth Token Refresh | [endpoint/004_oauth_token.md](endpoint/004_oauth_token.md) |
| endpoint | 005 | Claude CLI Roles | [endpoint/005_claude_cli_roles.md](endpoint/005_claude_cli_roles.md) |
| endpoint | 006 | Create API Key | [endpoint/006_create_api_key.md](endpoint/006_create_api_key.md) |
| endpoint | 007 | Metrics Enabled | [endpoint/007_metrics_enabled.md](endpoint/007_metrics_enabled.md) |
| endpoint | 008 | Shared Session Transcripts | [endpoint/008_shared_session_transcripts.md](endpoint/008_shared_session_transcripts.md) |
| endpoint | 009 | CLI Feedback | [endpoint/009_cli_feedback.md](endpoint/009_cli_feedback.md) |
| endpoint | 010 | Web Domain Info | [endpoint/010_web_domain_info.md](endpoint/010_web_domain_info.md) |

> `params` instances (73 files) use numbered naming (`001_action_mode.md`, `051_print.md`, …) and are enumerated in their master file: [params/readme.md](params/readme.md).
