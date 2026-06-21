# Tool: Monitor

Run a background command and feed output lines back to the model.

### Category

Background Tasks

### Permission Required

Yes (same rules as Bash)

### Description

Runs a command in the background and feeds each output line back to the model so
it can react to log entries, file changes, or polled status mid-conversation.
Use cases include tailing log files, polling CI/PR status, and watching
directories for changes.

### Availability

Not available on Bedrock, Vertex AI, Foundry, or when telemetry is disabled.

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `command` | string | yes | Shell command to run in the background |

### Since

v2.1.98 (2026-04-09)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master tool table |
| doc | [004_bash.md](004_bash.md) | Shell execution (non-background) |
