# Claude Code: Built-in Tools

All built-in tools available in Claude Code sessions. One file per tool in this directory.

### Scope

- **Purpose**: Authoritative reference for every built-in tool Claude Code exposes to the model.
- **Responsibility**: Master table and per-tool detail files.
- **In Scope**: All 40 built-in tools — file operations, shell, search, agents, tasks, scheduling, web, mode, interaction, MCP resources, publishing, notification, and extensibility tools.
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
| 017_task_output.md | Read task output (deprecated) |
| 018_task_stop.md | Stop background tasks |
| 019_task_update.md | Update task status |
| 020_cron_create.md | Create scheduled tasks |
| 021_cron_delete.md | Delete scheduled tasks |
| 022_cron_list.md | List scheduled tasks |
| 023_enter_plan_mode.md | Enter plan mode |
| 024_exit_plan_mode.md | Exit plan mode |
| 025_enter_worktree.md | Enter git worktree isolation |
| 026_exit_worktree.md | Exit git worktree isolation |
| 027_todo_write.md | Create and manage task lists |
| 028_artifact.md | Publish shareable artifacts |
| 029_monitor.md | Background command watcher |
| 030_powershell.md | Execute PowerShell commands |
| 031_push_notification.md | Desktop/phone notifications |
| 032_list_mcp_resources.md | List MCP server resources |
| 033_read_mcp_resource.md | Read MCP resource by URI |
| 034_remote_trigger.md | Manage Routines on claude.ai |
| 035_schedule_wakeup.md | Reschedule /loop iterations |
| 036_send_message.md | Agent team messaging |
| 037_share_onboarding_guide.md | Share team onboarding guide |
| 038_tool_search.md | Search/load deferred MCP tools |
| 039_wait_for_mcp_servers.md | Wait for MCP server connections |
| 040_workflow.md | Dynamic multi-subagent workflows |

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
| 17 | [TaskOutput](017_task_output.md) | Background Tasks | Read output from a background task **(deprecated v2.1.81)** |
| 18 | [TaskStop](018_task_stop.md) | Background Tasks | Stop a running background task |
| 19 | [TaskUpdate](019_task_update.md) | Background Tasks | Update status of a background task |
| 20 | [CronCreate](020_cron_create.md) | Scheduling | Create recurring scheduled tasks |
| 21 | [CronDelete](021_cron_delete.md) | Scheduling | Delete scheduled tasks |
| 22 | [CronList](022_cron_list.md) | Scheduling | List scheduled tasks |
| 23 | [EnterPlanMode](023_enter_plan_mode.md) | Mode | Enter plan mode (read-only analysis) |
| 24 | [ExitPlanMode](024_exit_plan_mode.md) | Mode | Exit plan mode |
| 25 | [EnterWorktree](025_enter_worktree.md) | Mode | Enter git worktree isolation |
| 26 | [ExitWorktree](026_exit_worktree.md) | Mode | Exit git worktree isolation |
| 27 | [TodoWrite](027_todo_write.md) | Interaction | Create and manage structured task lists |
| 28 | [Artifact](028_artifact.md) | Publishing | Publish HTML/Markdown as shareable artifact |
| 29 | [Monitor](029_monitor.md) | Background Tasks | Run background command, feed output to model |
| 30 | [PowerShell](030_powershell.md) | Shell | Execute PowerShell commands natively |
| 31 | [PushNotification](031_push_notification.md) | Notification | Send desktop or phone push notification |
| 32 | [ListMcpResourcesTool](032_list_mcp_resources.md) | MCP Resources | List resources from connected MCP servers |
| 33 | [ReadMcpResourceTool](033_read_mcp_resource.md) | MCP Resources | Read a specific MCP resource by URI |
| 34 | [RemoteTrigger](034_remote_trigger.md) | Scheduling | Create/run Routines on claude.ai |
| 35 | [ScheduleWakeup](035_schedule_wakeup.md) | Scheduling | Reschedule next /loop iteration |
| 36 | [SendMessage](036_send_message.md) | Agents | Message agent team teammate or subagent |
| 37 | [ShareOnboardingGuide](037_share_onboarding_guide.md) | Interaction | Upload ONBOARDING.md, return share link |
| 38 | [ToolSearch](038_tool_search.md) | Extensibility | Search/load deferred MCP tool schemas |
| 39 | [WaitForMcpServers](039_wait_for_mcp_servers.md) | Extensibility | Wait for background MCP server connections |
| 40 | [Workflow](040_workflow.md) | Agents | Run dynamic multi-subagent workflow |

### Availability Notes

Not all tools are available in all versions or configurations:

- **Task tools (14–16, 19)**: TaskCreate, TaskGet, TaskList, TaskUpdate are the
  default since v2.1.142. Set `CLAUDE_CODE_ENABLE_TASKS=0` to revert to TodoWrite.
- **TaskOutput (17)**: Deprecated since v2.1.81. Use `Read` on the task's output
  file path instead.
- **TodoWrite (27)**: Disabled by default since v2.1.142 (superseded by Task tools).
  Set `CLAUDE_CODE_ENABLE_TASKS=0` to re-enable.
- **PowerShell (30)**: Auto-enabled on Windows without Git Bash. Opt-in on
  Linux/macOS via `CLAUDE_CODE_USE_POWERSHELL_TOOL=1`.
- **SendMessage (36)**: Requires `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1`.
- **ToolSearch (38)** / **WaitForMcpServers (39)**: Mutually exclusive — ToolSearch
  appears when MCP tool search is enabled, WaitForMcpServers when disabled.
- **Artifact (28)**, **RemoteTrigger (34)**, **ScheduleWakeup (35)**,
  **ShareOnboardingGuide (37)**: Require specific subscription plans; not available
  on Bedrock/Vertex/Foundry.
- **Monitor (29)**: Not available on Bedrock/Vertex/Foundry or with telemetry
  disabled.

The tool set exposed to the model depends on `--tools`, `--allowed-tools`, and
`--disallowed-tools` parameters.

### Categories

| Category | Tools | Count |
|----------|-------|------:|
| File Operations | Read, Write, Edit, NotebookEdit | 4 |
| Shell | Bash, PowerShell | 2 |
| Search | Glob, Grep | 2 |
| Agents | Agent, SendMessage, Workflow | 3 |
| Interaction | AskUserQuestion, TodoWrite, ShareOnboardingGuide | 3 |
| Web | WebFetch, WebSearch | 2 |
| Code Intelligence | LSP | 1 |
| Extensibility | Skill, ToolSearch, WaitForMcpServers | 3 |
| Background Tasks | TaskCreate, TaskGet, TaskList, TaskOutput, TaskStop, TaskUpdate, Monitor | 7 |
| Scheduling | CronCreate, CronDelete, CronList, RemoteTrigger, ScheduleWakeup | 5 |
| Mode | EnterPlanMode, ExitPlanMode, EnterWorktree, ExitWorktree | 4 |
| MCP Resources | ListMcpResourcesTool, ReadMcpResourceTool | 2 |
| Publishing | Artifact | 1 |
| Notification | PushNotification | 1 |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [../param/068_tools.md](../param/068_tools.md) | `--tools` parameter for overriding available tool set |
| doc | [../param/006_allowed_tools.md](../param/006_allowed_tools.md) | `--allowed-tools` for tool allowlisting |
| doc | [../param/022_disallowed_tools.md](../param/022_disallowed_tools.md) | `--disallowed-tools` for tool denylisting |
