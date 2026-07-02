# Tool: SendMessage

Send a message to an agent team teammate or resume a subagent.

### Category

Agents

### Permission Required

No

### Description

Sends a message to an agent team teammate, or resumes a subagent by its agent ID.
Only available when experimental agent teams are enabled via
`CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1`.

### Availability

Requires `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1` environment variable.

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `agentId` | string | yes | Target agent or teammate ID |
| `message` | string | yes | Message content to send |

### Since

v2.1.139+ (unverified) — agent teams research preview

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master tool table |
| doc | [007_agent.md](007_agent.md) | Launch subagent processes |
| doc | [../param/080_experimental_agent_teams.md](../param/080_experimental_agent_teams.md) | Agent teams env var |
