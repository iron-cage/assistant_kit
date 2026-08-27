# Format: Task

### Scope

- **Purpose**: Specify the `~/.claude/tasks/{session-uuid}/{n}.json` format — one JSON object per task, in a per-session directory.
- **Responsibility**: Authoritative instance for task JSON format — directory layout, task object fields, status values, the dependency-graph fields, and the lock file.
- **In Scope**: File location and layout, per-task object schema, `id`/`subject`/`description`/`status`/`blocks`/`blockedBy`/`activeForm`/`metadata`/`owner` fields, status values, `.lock`.
- **Out of Scope**: Task directory context (→ [`../storage/002_support_directories.md`](../storage/002_support_directories.md)); the tools that write this format (→ [`../tool/014_task_create.md`](../tool/014_task_create.md), [`../tool/019_task_update.md`](../tool/019_task_update.md)).

### Supersession

> ⚠️ **This format replaced an earlier one and the change is total** — path, granularity,
> container type, and field names all differ. Both are recorded because consumers written
> against the old shape fail silently rather than loudly: a reader that globs
> `~/.claude/todos/*.json` simply finds nothing and reports zero tasks.

| | Superseded | Current (v2.1.220) |
|---|---|---|
| Path | `~/.claude/todos/{session-uuid}.json` | `~/.claude/tasks/{session-uuid}/{n}.json` |
| Granularity | One file per session | One file **per task**, numbered from `1` |
| Container | JSON **array** of task objects | JSON **object** — a single task |
| Identity | positional (array index) | explicit `id` field |
| Text fields | `content` + `activeForm` | `subject` + `description` (+ `activeForm`, now optional) |
| Dependencies | none | `blocks` / `blockedBy` |
| Concurrency | none | `.lock` file per session directory |

`content` does not occur in any of the 17822 task files surveyed. See
[`../storage/002_support_directories.md`](../storage/002_support_directories.md) § Rename
for the binary evidence that `todos/` → `tasks/` was a rename with a read-compatibility
shim (`"tasks" in e || "todos" in e`) rather than two coexisting directories.

### Location

`~/.claude/tasks/{session-uuid}/{n}.json`

**Format**: one JSON object per file — a single task, pretty-printed with 2-space indent.
**Numbering**: `{n}` is a decimal counter starting at `1`, unpadded (`1.json`, `2.json`, …
`27.json`). It matches the task's own `id` field, which is a **string** not a number.
**Mutability**: individual task files are rewritten on status change; sibling tasks are
untouched. This is the practical gain over the old whole-array overwrite.
**Sibling**: `.lock` — present in all 250 session directories surveyed, always 0 bytes.
Presence is the signal; there is no content to parse.

### Schema

```json
{
  "id": "27",
  "subject": "PR 117 round-11 /pr_review pipeline (submission)",
  "description": "Execute the full /pr_review Steps 0-17 pipeline for PR 117, round 11 artifact dir (3315_..._pr117__r11), targeting GitHub review submission.",
  "status": "completed",
  "blocks": [],
  "blockedBy": []
}
```

### Task Object Fields

Frequencies from a census of 17822 task files across 250 session directories, 2026-08-27:

| Field | Count | Type | Required | Meaning |
|-------|-------|------|----------|---------|
| `id` | 17822 | string | ✅ | Task identity; matches the filename stem. A **string**, even though it is always digits |
| `subject` | 17822 | string | ✅ | Short task title |
| `description` | 17822 | string | ✅ | Full task description; may be empty but the key is always present |
| `status` | 17822 | string | ✅ | `"pending"`, `"in_progress"`, or `"completed"` |
| `blocks` | 17822 | array | ✅ | IDs of tasks this one blocks |
| `blockedBy` | 17822 | array | ✅ | IDs of tasks blocking this one |
| `activeForm` | 12662 | string | ❌ (71%) | Present-continuous phrasing, carried over from the old format |
| `metadata` | 99 | object | ❌ (0.6%) | Contents not characterized |
| `owner` | 1 | — | ❌ (0.006%) | Single occurrence; contents not characterized |

**Six fields are universal, three are not.** Treat `activeForm` as optional despite its
being mandatory in the superseded format — 29% of real task files omit it, so a consumer
requiring it will fail on nearly a third of real data.

`blocks`/`blockedBy` make the task set a directed graph, not a flat list. Both were empty
in the sampled records; whether the binary enforces acyclicity is ❓ Uncertain — not tested.

### Status Distribution

| Status | Count | Share |
|--------|-------|-------|
| `completed` | 17534 | 98.4% |
| `pending` | 197 | 1.1% |
| `in_progress` | 93 | 0.5% |

No fourth status value occurs. The three match the `status` enum documented for
[`../tool/019_task_update.md`](../tool/019_task_update.md).

### File Organization

One **directory** per session UUID, containing one numbered `.json` per task plus `.lock`.
The directory is created when the first task is written for a session; absent if none were.

Re-derive everything above:

```bash
cd ~/.claude/tasks   # relative root — an absolute path can silently return nothing

find . -maxdepth 1 -type d | tail -n +2 | wc -l                    # session directories
find . -name '*.json' | wc -l                                       # task files
find . -name '*.json' -exec grep -ho '^  "[a-zA-Z]*"' {} + \
  | tr -d ' "' | sort | uniq -c | sort -rn                          # field frequency
find . -name '*.json' -exec grep -ho '"status":[[:space:]]*"[^"]*"' {} + \
  | sed 's/.*"\([^"]*\)"$/\1/' | sort | uniq -c | sort -rn          # status distribution
find . -name '*.json' -exec grep -l '"content"' {} + | wc -l        # → 0, the old field is gone
```

The field-frequency command relies on the 2-space pretty-printing (`^  "key"`), which is
how these files are actually written — it would miss keys in a minified object. The final
command is the check that matters most: a non-zero result would mean both formats coexist.

### Since

`tasks/` layout: unverified — present in v2.1.220, and no release note in
[`../version/`](../version/readme.md) mentions either directory name, so the introducing
version is not established. The superseded `todos/` layout dates to pre-v1.0.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Formats master index |
| storage | [`../storage/002_support_directories.md`](../storage/002_support_directories.md) | `tasks/` directory: organization, and the `todos/` → `tasks/` rename evidence |
| tool | [`../tool/014_task_create.md`](../tool/014_task_create.md) | TaskCreate — writes a new numbered task file |
| tool | [`../tool/019_task_update.md`](../tool/019_task_update.md) | TaskUpdate — rewrites one task file in place |
| tool | [`../tool/016_task_list.md`](../tool/016_task_list.md) | TaskList — reads the session directory |
| tool | [`../tool/027_todo_write.md`](../tool/027_todo_write.md) | TodoWrite — still exposes `content`/`activeForm` parameters, which no longer match this on-disk schema |
