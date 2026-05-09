# Task Operations

- **Actor:** Developer
- **Trigger:** A task is created, progresses, or closes for this crate.
- **Emits:** —

## Add Task

1. Coordinate with the workspace root `task/readme.md` to obtain the next global task ID
2. Create `NNN_{snake_case_title}.md` in `task/` with the task specification
3. Register in `readme.md` Active Tasks table: add row with ID, Status, Title, Category, Created date, File link

## Complete Task

1. Move task file from `task/` to `task/completed/`
2. Update `readme.md` Active Tasks table: set Status to complete, update File link to `completed/NNN_*.md`

## Backlog Task

1. Move task file from `task/` to `task/backlog/`
2. Update `readme.md` Active Tasks table: set Status to backlogged, update File link to `backlog/NNN_*.md`
