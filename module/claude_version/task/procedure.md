# Task Lifecycle Procedures

- **Actor:** Developer
- **Trigger:** A task is created, transitions state, or is closed.
- **Emits:** —

## Create Task

1. Confirm the task doesn't duplicate an existing open task (`task/readme.md` Tasks Index)
2. Assign the next 3-digit ID from `task/readme.md` frontmatter `next_id`; increment `next_id` by 1
3. Create `NNN_{snake_case_description}.md` in `task/unverified/`
4. Register in `task/readme.md` Tasks Index: add row with ID, state ❓ Unverified, Executor, Dir, Task link, Purpose
5. Verify the task is actionable and well-scoped before advancing to Verified

## Transition Task State

1. Move the task file to the directory matching the new state (`unverified/` → `verifying/` → `executing/` → `validating/`)
2. Update the task file's state line to the new emoji and label
3. Update the `task/readme.md` Tasks Index row: set new State value

## Close Task

1. Move the task file to `completed/` (success) or `cancelled/` (abandoned)
2. Update the task file's state to ✅ Closed (completed) or ❌ Closed (cancelled)
3. Update the `task/readme.md` Tasks Index row: set State to ✅ (Closed) or ❌ (Closed)
