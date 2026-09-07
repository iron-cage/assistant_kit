# Tool: Workflow

Run a dynamic workflow orchestrating multiple subagents.

### Category

Agents

### Permission Required

Yes

### Description

Runs a dynamic workflow — a script that orchestrates many subagents in the
background and returns one consolidated result. Workflows allow complex
multi-step operations to be expressed as a single tool call with parallel
subagent execution.

### Availability

Requires `CLAUDE_CODE_DISABLE_WORKFLOWS` to not be set.

### Parameters

❌ **`workflow: string` refuted** — the doc previously claimed a single, oversimplified required `workflow` parameter. Confirmed wrong: v2.1.220's actual schema has no field named `workflow` at all; it takes a `script`/`scriptPath`/`name` triad (at least one required, per the schema's own `.refine()`) plus four more optional fields:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `script` | string | one of script/scriptPath/name | Self-contained workflow script (max ~512KB), starting with `export const meta = {name, description, phases}`, using `agent()`/`parallel()`/`pipeline()`/`phase()`/`log()`. |
| `scriptPath` | string | one of script/scriptPath/name | Path to a workflow script file on disk (e.g. a script Written earlier, or returned from a prior invocation). Takes precedence over `script` and `name`. |
| `name` | string | one of script/scriptPath/name | Name of a predefined workflow (built-in or from `.claude/workflows/`). Resolves to a self-contained script. |
| `args` | any | ❌ | Input value exposed to the script as the global `args`, verbatim — used to parameterize a named workflow. |
| `resumeFromRunId` | string | ❌ | Run ID (pattern `^wf_[a-z0-9-]{6,}$`) of a prior Workflow invocation to resume from. Completed `agent()` calls with an unchanged `(prompt, opts)` return cached results instantly; only edited/new calls re-run. Same-session only. |
| `title` | string | ❌ | Ignored — set the workflow title in the script's `meta` block instead. |
| `description` | string | ❌ | Ignored — set the workflow description in the script's `meta` block instead. |

**Verify:** `grep -ac -F "resumeFromRunId" ~/.local/share/claude/versions/2.1.220` → 17; the exact `.describe()` text for every field above (including the `—`-escaped "Ignored — set the workflow title/description..." pair, which a naive literal-em-dash grep will miss) is present verbatim in that same binary.

### Since

v2.1.153 (2026-05-28)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master tool table |
| doc | [007_agent.md](007_agent.md) | Individual subagent launch |
| doc | [036_send_message.md](036_send_message.md) | Message passing between agents |
