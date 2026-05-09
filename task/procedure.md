# Task Operations — dream Workspace

- **Actor:** Developer
- **Trigger:** A task is created, progresses, or closes in the dream workspace.
- **Emits:** —

## Add Task

1. Assign the next available ID (check `readme.md` Active Tasks table for the current highest ID, increment by 1)
2. Create `NNN_{snake_case_title}.md` in `task/` with the task specification
3. Register in `readme.md` Active Tasks table: add row with ID, Status (📥 Inbox), Title, Category, Created date, File link

## Complete Task

1. Move task file from `task/` to `task/completed/`
2. Update `readme.md` Active Tasks table: set Status to complete, update File link to `completed/NNN_*.md`

## Backlog Task

1. Move task file from `task/` to `task/backlog/`
2. Update `readme.md` Active Tasks table: set Status to backlogged, update File link to `backlog/NNN_*.md`

## Cancel Task

1. Move task file from `task/` to `task/completed/` (or delete if not worth archiving)
2. Update `readme.md` Active Tasks table: set Status to cancelled, update File link accordingly

## Example

Creating task `003_storage_refactor.md`:

1. Check `readme.md` Active Tasks — current highest ID is `002`
2. Create `003_storage_refactor.md` in `task/` with title, goal, and steps
3. Add row: `| 003 | 📥 | Storage Refactor | Architecture | 2026-05-10 | [003_storage_refactor.md](003_storage_refactor.md) |`
