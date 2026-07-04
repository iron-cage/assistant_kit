# Tool: SendMessage

Send a message to an agent team teammate or resume a subagent.

### Category

Agents

### Permission Required

No

### Description

Sends a message to another agent. Three distinct uses: resuming a previously
spawned background subagent by ID (general-purpose, no special configuration
required), a background subagent messaging back to the main conversation via
`to: "main"`, or messaging a named teammate in an agent team (requires
experimental agent teams).

### Availability

No special requirement for resuming a previously spawned subagent
(`to: <agentId>`) or for a background subagent messaging the main conversation
(`to: "main"`). Messaging a named teammate (`to: <teammate-name>`) requires
`CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1`.

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `to` | string | yes | Recipient: a teammate name, `"main"` (the main conversation; background subagents only), or a previously-spawned agent's ID (format `a...-...`) to resume it |
| `message` | string | yes | Plain text message content |
| `summary` | string | conditional | Required when `message` is a string; 5-10 word summary shown as a UI preview (max 200 chars) |

### Since

v2.1.32+ — agent-teams teammate messaging (research preview, requires `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1`); v2.1.77+ — general subagent-resume capability (`SendMessage({to: agentId})`, replaces the Agent tool's removed `resume` parameter, no flag required)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master tool table |
| doc | [007_agent.md](007_agent.md) | Launch subagent processes |
| doc | [../param/080_experimental_agent_teams.md](../param/080_experimental_agent_teams.md) | Agent teams env var |
