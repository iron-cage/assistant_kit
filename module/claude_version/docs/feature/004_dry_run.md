# Feature: Dry Run

### Scope

- **Purpose**: Document the dry-run preview mode available on all mutation commands.
- **Responsibility**: Describe dry::1 flag semantics, output parity requirement, and affected commands.
- **In Scope**: dry::1 parameter, [dry-run] output prefix, parity with actual action messages, affected commands (.version.install, .version.guard, .processes.kill, .settings.set).
- **Out of Scope**: Individual command behavior under normal execution (→ `feature/001_version_management.md`, `feature/002_process_lifecycle.md`, `feature/003_settings_management.md`).

### Design

`dry::1` is available on all mutation commands. When set, the command prints its intended action without executing any side effects.

**Affected commands:**
- `.version.install dry::1`
- `.version.guard dry::1`
- `.processes.kill dry::1`
- `.settings.set dry::1`

**Output parity requirement:** The `[dry-run] would ...` output must exactly mirror the actual action message produced without `dry::1`. Both modes share identical argument extraction logic to prevent divergence over time. If the actual action says `"Installing version 2.1.78"`, the dry-run must say `"[dry-run] would: Installing version 2.1.78"`.

**Precedence:** `dry::1` takes precedence over `force::1` when both are specified. A dry-run with `force::1` shows what a force operation would do without executing it.

**Note:** `dry::1` on read-only commands (`.settings.show`, `.version.show`, etc.) is not supported — those commands have no mutations to preview.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [feature/001_version_management.md](001_version_management.md) | Version mutation commands supporting dry::1 |
| doc | [feature/002_process_lifecycle.md](002_process_lifecycle.md) | Kill command supporting dry::1 |
| doc | [feature/003_settings_management.md](003_settings_management.md) | Settings set command supporting dry::1 |
| source | `../../src/commands.rs` | dry-run branches in mutation routines |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | FR-05, Command Inventory (dry:: parameter), Parameter Inventory (dry:: — Boolean 0/1) |
