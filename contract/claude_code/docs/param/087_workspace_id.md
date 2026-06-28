# workspace_id

Specifies the Anthropic workspace ID for API requests.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `ANTHROPIC_WORKSPACE_ID` |
| Config Key | — |

### Type

string (UUID)

### Default

— (none; uses default workspace)

### Since

v2.1.141

### Description

Sets the Anthropic workspace ID for routing API requests. When set, all API
calls include this workspace identifier, enabling workspace-level billing,
rate limits, and access controls. Used by organizations with multiple
Anthropic workspaces.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [007_api_key.md](007_api_key.md) | API key authentication |
