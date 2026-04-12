# Doc Entities

## Master Doc Entities Table

| Type | Purpose | Master File | Instances |
|------|---------|-------------|----------:|
| `feature/` | Workspace design and crate inventory | [feature/readme.md](feature/readme.md) | 2 |
| `invariant/` | Privacy, versioning, testing, and performance constraints | [invariant/readme.md](invariant/readme.md) | 4 |
| `pattern/` | Four-layer crate dependency architecture | [pattern/readme.md](pattern/readme.md) | 1 |
| `integration/` | Cross-workspace integration protocol | [integration/readme.md](integration/readme.md) | 1 |
| `error/` | Claude Code error message catalog | [error/readme.md](error/readme.md) | 5 |
| `claude_code/` | External Claude Code binary behavior, storage formats, filesystem conventions | [claude_code/readme.md](claude_code/readme.md) | 7 |

## Master Doc Instances Table

| Entity | ID | Name | File |
|--------|----|------|------|
| feature | 001 | Workspace Design | [feature/001_workspace_design.md](feature/001_workspace_design.md) |
| feature | 002 | Agent Inventory | [feature/002_agent_inventory.md](feature/002_agent_inventory.md) |
| invariant | 001 | Privacy Invariant | [invariant/001_privacy_invariant.md](invariant/001_privacy_invariant.md) |
| invariant | 002 | Versioning Strategy | [invariant/002_versioning_strategy.md](invariant/002_versioning_strategy.md) |
| invariant | 003 | Testing Strategy | [invariant/003_testing_strategy.md](invariant/003_testing_strategy.md) |
| invariant | 004 | Performance | [invariant/004_performance.md](invariant/004_performance.md) |
| pattern | 001 | Crate Layering | [pattern/001_crate_layering.md](pattern/001_crate_layering.md) |
| integration | 001 | Consumer Workspace Integration | [integration/001_consumer_integration.md](integration/001_consumer_integration.md) |
| error | 001 | Rate Limit Reached | [error/001_rate_limit_reached.md](error/001_rate_limit_reached.md) |
| error | 002 | Authentication Failed | [error/002_authentication_failed.md](error/002_authentication_failed.md) |
| error | 003 | Context Limit Reached | [error/003_context_limit_reached.md](error/003_context_limit_reached.md) |
| error | 004 | Request Timed Out | [error/004_request_timed_out.md](error/004_request_timed_out.md) |
| error | 005 | API Overloaded | [error/005_api_overloaded.md](error/005_api_overloaded.md) |
| claude_code | 001 | Session Behaviors | [claude_code/001_session_behaviors.md](claude_code/001_session_behaviors.md) |
| claude_code | 002 | Storage Organization | [claude_code/002_storage_organization.md](claude_code/002_storage_organization.md) |
| claude_code | 003 | Filesystem Layout | [claude_code/003_filesystem_layout.md](claude_code/003_filesystem_layout.md) |
| claude_code | 004 | JSONL Format | [claude_code/004_jsonl_format.md](claude_code/004_jsonl_format.md) |
| claude_code | 005 | Settings Format | [claude_code/005_settings_format.md](claude_code/005_settings_format.md) |
| claude_code | 006 | Ancillary Formats | [claude_code/006_ancillary_formats.md](claude_code/006_ancillary_formats.md) |
| claude_code | params | Parameters | [claude_code/params/readme.md](claude_code/params/readme.md) |
