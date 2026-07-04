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
| `action` | string (enum) | yes | One of: `list`, `get`, `create`, `update`, `run` |
| `trigger_id` | string | conditional | Required for `get`, `update`, and `run` |
| `body` | object | conditional | Required for `create` and `update`; optional for `run` |

### Since

pre-2.1.101 (unverified) — confirmed to exist by v2.1.101 (bugfix reference); no changelog line documents its introduction

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master tool table |
| doc | [020_cron_create.md](020_cron_create.md) | Local scheduled tasks (create) |
| doc | [021_cron_delete.md](021_cron_delete.md) | Local scheduled tasks (delete) |
| doc | [022_cron_list.md](022_cron_list.md) | Local scheduled tasks (list) |
| doc | [035_schedule_wakeup.md](035_schedule_wakeup.md) | Loop iteration scheduling |
