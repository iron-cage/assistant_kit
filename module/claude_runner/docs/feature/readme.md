# Feature Doc Entity

### Scope

- **Purpose**: Document user-facing capabilities of the claude_runner crate for CLI users and automation consumers.
- **Responsibility**: Index of feature doc instances covering the clr binary tool design.
- **In Scope**: Execution modes, default flags, YAML library surface, CLI flag behavior, forwarding a prompt to other topics.
- **Out of Scope**: Dependency constraints (→ `invariant/`), public API contracts (→ `api/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Runner Tool](001_runner_tool.md) | clr binary design: modes, default flags, and YAML library | ✅ |
| 002 | [Journaling Integration](002_journaling_integration.md) | Integration with claude_journal for automatic event journaling | ✅ |
| 003 | [Retry Hierarchy](003_retry_hierarchy.md) | 3-tier retry resolution for 6 error classes | ✅ |
| 004 | [JSON Config Loading](004_json_config.md) | JSON file and stdin pipe loading for all clr parameters | ✅ |
| 005 | [Session Path Resolution](005_session_path_resolution.md) | `scope_for()`, 6 CLAUDE_* variables, `--from`, `clr scope` command | ✅ |
| 006 | [CLI Design](006_cli_design.md) | `--flag value` syntax rationale, parser design, and flag-level decisions | ✅ |
| 007 | [YAML Global Config](007_yaml_global_config.md) | YAML config files with profile support and cross-subcommand scope | 🔄 |
| 008 | [Interactive Handoff](008_interactive_handoff.md) | Releasing the daemon's session before opening it on the caller's terminal | 📋 |
| 009 | [Topic Forwarding](009_topic_forwarding.md) | Sending one prompt to one topic (`delegate`) or to every live one (`broadcast`), and provisioning the topics they address (`pool`) | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating feature doc instances | ✅ |

**Status:** ✅ implemented · 🔄 in progress · 📋 specified, not yet built.
