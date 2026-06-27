# Tool: TaskOutput (Deprecated)

Read output from a background task.

**Deprecated since v2.1.81.** Prefer using the `Read` tool on the task's output
file path instead. The session-scoped in-memory registry is cleared on context
compaction, making this tool structurally unreliable.

### Category

Background Tasks

### Description

Reads the output produced by a background task. Can optionally block until the
task completes. Session-scoped in-memory registry is cleared on context
compaction, which means task IDs become invalid after compaction events.

### Since

pre-v1.0 (unverified); deprecated v2.1.81 (2026-03-20)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master tool table |
| doc | [001_read.md](001_read.md) | Read — replacement: use on task output file path |
| doc | [015_task_get.md](015_task_get.md) | TaskGet — use to get task status instead |
