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

❌ **`delayMs` refuted** — the doc previously named the parameter `delayMs` with a 60000–3600000 millisecond range. Confirmed wrong: v2.1.220's real parameter is `delaySeconds`, a **seconds**-denominated field clamped to `[60, 3600]` — same numeric range as claimed, wrong unit and wrong name (binary: `delaySeconds` 15 hits vs `delayMs` 4 hits, the latter attributable to unrelated generic identifier noise).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `delaySeconds` | number | unless `stop: true` | Seconds from now to wake up. Clamped to `[60, 3600]` by the runtime. |
| `prompt` | string | unless `stop: true` | The `/loop` input to fire on wake-up; pass the same prompt back each turn so the next firing repeats the task. |
| `reason` | string | unless `stop: true` | One short, specific sentence explaining the chosen delay — goes to telemetry and is shown to the user. |
| `stop` | boolean | ❌ | Set `true` to end the dynamic loop immediately instead of scheduling another wakeup. When `true`, all other fields are ignored. |
| `noop` | boolean | unless `stop: true` | `true` = nothing changed this tick (collapsed in the user's terminal view); `false` = something happened worth keeping. Omit when stopping. |

### Since

v2.0+ (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master tool table |
| doc | [034_remote_trigger.md](034_remote_trigger.md) | Routine management |
| doc | [020_cron_create.md](020_cron_create.md) | Local scheduled tasks |
