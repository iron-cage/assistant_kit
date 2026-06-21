# Tool: TodoWrite

Create and manage structured task lists for tracking progress.

### Category

Interaction

### Description

Creates and manages a structured todo list visible to the user during the session. Each todo item has a content description (imperative form), an activeForm description (present continuous form shown during execution), and a status (pending, in_progress, completed). The tool replaces the entire todo list on each invocation — callers must include all items, not just changed ones. Exactly one item should be in_progress at a time. Used proactively for multi-step tasks to demonstrate progress and help users track work. Items should be marked completed immediately after finishing, not batched.

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `todos` | array | yes | Complete updated todo list |
| `todos[].content` | string | yes | Task description in imperative form (e.g., "Run tests") |
| `todos[].activeForm` | string | yes | Task description in present continuous (e.g., "Running tests") |
| `todos[].status` | enum | yes | `pending`, `in_progress`, or `completed` |

### Since

v0.2.93 (2025-04-30); disabled by default since v2.1.142 (superseded by Task tools)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master tool table |
