# Tool: RemoteTrigger

Create, update, run, and list Routines on claude.ai.

### Category

Scheduling

### Permission Required

No

### Description

Manages Routines — scheduled automation that runs on Anthropic infrastructure.
Backs the `/schedule` command. Can create new routines, update existing ones,
trigger immediate runs, and list all configured routines.

### Availability

Requires Pro, Max, Team, or Enterprise plan. Not available on Bedrock, Vertex AI,
or Foundry.

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `action` | string | yes | Action to perform: create, update, run, or list |

### Since

v2.0+ (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master tool table |
| doc | [020_cron_create.md](020_cron_create.md) | Local scheduled tasks |
| doc | [035_schedule_wakeup.md](035_schedule_wakeup.md) | Loop iteration scheduling |
