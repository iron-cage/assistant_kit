# Claude Code: Built-in Tools

All built-in tools available in Claude Code sessions. One file per tool in this directory.

### Scope

- **Purpose**: Authoritative reference for every built-in tool Claude Code exposes to the model.
- **Responsibility**: Master table and per-tool detail files.
- **In Scope**: All 26 built-in tools — file operations, shell, search, agents, tasks, scheduling, web, mode, and utility tools.
- **Out of Scope**: MCP tools (user-installed extensions), custom agent tools, IDE-specific tools.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| readme.md | Master tool table (this file) |
| 001_read.md | Read files from filesystem |
| 002_write.md | Write/create files |
| 003_edit.md | Patch existing files via string replacement |
| 004_bash.md | Execute shell commands |
| 005_glob.md | File pattern matching |
| 006_grep.md | Content search (ripgrep) |
| 007_agent.md | Launch subagent processes |
| 008_ask_user_question.md | Prompt user for input |
| 009_web_fetch.md | Fetch URL content |
| 010_web_search.md | Web search |
| 011_notebook_edit.md | Edit Jupyter notebooks |
| 012_lsp.md | Language server protocol queries |
| 013_skill.md | Invoke user-defined skills |
| 014_task_create.md | Create background tasks |
| 015_task_get.md | Get task information |
| 016_task_list.md | List background tasks |
| 017_task_output.md | Read task output |
| 018_task_stop.md | Stop background tasks |
| 019_task_update.md | Update task status |
| 020_cron_create.md | Create scheduled tasks |
| 021_cron_delete.md | Delete scheduled tasks |
| 022_cron_list.md | List scheduled tasks |
| 023_enter_plan_mode.md | Enter plan mode |
| 024_exit_plan_mode.md | Exit plan mode |
| 025_enter_worktree.md | Enter git worktree isolation |
| 026_exit_worktree.md | Exit git worktree isolation |

### Tool Table

| # | Tool Name | Category | Description |
|---|-----------|----------|-------------|
| 1 | [Read](001_read.md) | File Operations | Read files (text, image, PDF, notebook) with line numbers |
| 2 | [Write](002_write.md) | File Operations | Create or overwrite files |
| 3 | [Edit](003_edit.md) | File Operations | Patch files via exact string replacement |
| 4 | [Bash](004_bash.md) | Shell | Execute shell commands with timeout control |
| 5 | [Glob](005_glob.md) | Search | Find files by glob patterns |
| 6 | [Grep](006_grep.md) | Search | Search file contents with regex (ripgrep) |
| 7 | [Agent](007_agent.md) | Agents | Launch specialized subagent processes |
| 8 | [AskUserQuestion](008_ask_user_question.md) | Interaction | Prompt user for input or clarification |
| 9 | [WebFetch](009_web_fetch.md) | Web | Fetch content from a URL |
| 10 | [WebSearch](010_web_search.md) | Web | Search the web |
| 11 | [NotebookEdit](011_notebook_edit.md) | File Operations | Edit Jupyter notebook cells |
| 12 | [LSP](012_lsp.md) | Code Intelligence | Language server protocol queries |
| 13 | [Skill](013_skill.md) | Extensibility | Invoke user-defined slash command skills |
| 14 | [TaskCreate](014_task_create.md) | Background Tasks | Create and start background tasks |
| 15 | [TaskGet](015_task_get.md) | Background Tasks | Get information about a background task |
| 16 | [TaskList](016_task_list.md) | Background Tasks | List all background tasks |
| 17 | [TaskOutput](017_task_output.md) | Background Tasks | Read output from a background task |
| 18 | [TaskStop](018_task_stop.md) | Background Tasks | Stop a running background task |
| 19 | [TaskUpdate](019_task_update.md) | Background Tasks | Update status of a background task |
| 20 | [CronCreate](020_cron_create.md) | Scheduling | Create recurring scheduled tasks |
| 21 | [CronDelete](021_cron_delete.md) | Scheduling | Delete scheduled tasks |
| 22 | [CronList](022_cron_list.md) | Scheduling | List scheduled tasks |
| 23 | [EnterPlanMode](023_enter_plan_mode.md) | Mode | Enter plan mode (read-only analysis) |
| 24 | [ExitPlanMode](024_exit_plan_mode.md) | Mode | Exit plan mode |
| 25 | [EnterWorktree](025_enter_worktree.md) | Mode | Enter git worktree isolation |
| 26 | [ExitWorktree](026_exit_worktree.md) | Mode | Exit git worktree isolation |

### Categories

| Category | Tools | Count |
|----------|-------|------:|
| File Operations | Read, Write, Edit, NotebookEdit | 4 |
| Shell | Bash | 1 |
| Search | Glob, Grep | 2 |
| Agents | Agent | 1 |
| Interaction | AskUserQuestion | 1 |
| Web | WebFetch, WebSearch | 2 |
| Code Intelligence | LSP | 1 |
| Extensibility | Skill | 1 |
| Background Tasks | TaskCreate, TaskGet, TaskList, TaskOutput, TaskStop, TaskUpdate | 6 |
| Scheduling | CronCreate, CronDelete, CronList | 3 |
| Mode | EnterPlanMode, ExitPlanMode, EnterWorktree, ExitWorktree | 4 |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [../params/068_tools.md](../params/068_tools.md) | `--tools` parameter for overriding available tool set |
| doc | [../params/006_allowed_tools.md](../params/006_allowed_tools.md) | `--allowed-tools` for tool allowlisting |
| doc | [../params/022_disallowed_tools.md](../params/022_disallowed_tools.md) | `--disallowed-tools` for tool denylisting |
