# Tool: Workflow

Run a dynamic workflow orchestrating multiple subagents.

### Category

Agents

### Permission Required

Yes

### Description

Runs a dynamic workflow — a script that orchestrates many subagents in the
background and returns one consolidated result. Workflows allow complex
multi-step operations to be expressed as a single tool call with parallel
subagent execution.

### Availability

Requires `CLAUDE_CODE_DISABLE_WORKFLOWS` to not be set.

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `workflow` | string | yes | Workflow definition or identifier |

### Since

v2.1.153 (2026-05-28)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master tool table |
| doc | [007_agent.md](007_agent.md) | Individual subagent launch |
