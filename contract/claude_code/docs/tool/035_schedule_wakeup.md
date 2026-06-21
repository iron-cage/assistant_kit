# Tool: ScheduleWakeup

Reschedule the next iteration of a self-paced `/loop`.

### Category

Scheduling

### Permission Required

No

### Description

Called by the model at the end of each `/loop` iteration to pick when the next
iteration runs. Interval can range from 1 minute to 1 hour. Not called directly
by users — the model invokes this tool automatically during loop execution.

### Availability

Not available on Bedrock, Vertex AI, or Foundry.

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `delayMs` | number | yes | Delay in milliseconds before next iteration (60000–3600000) |

### Since

v2.0+ (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master tool table |
| doc | [034_remote_trigger.md](034_remote_trigger.md) | Routine management |
| doc | [020_cron_create.md](020_cron_create.md) | Local scheduled tasks |
