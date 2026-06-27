# api_key

The Anthropic API key used to authenticate all API requests.

### Forms

| | Value |
|-|-------|
| CLI Flag | — (removed from CLI; was `--api-key`) |
| Env Var | `ANTHROPIC_API_KEY` |
| Config Key | — |

### Type

string

### Default

—

### Since

pre-v1.0 (unverified)

### Description

The Anthropic API key used to authenticate requests. Must be set in the environment; the `--api-key` CLI flag was removed from the binary. Without a valid key, Claude Code falls back to browser-based OAuth authentication. For automation and CI, always set this env var explicitly rather than relying on interactive login.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [087_workspace_id.md](087_workspace_id.md) | Workspace ID for API routing |