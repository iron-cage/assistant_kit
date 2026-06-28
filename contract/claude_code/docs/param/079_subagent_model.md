# subagent_model

Sets the model used for background subagent sessions spawned by the Agent tool.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CODE_SUBAGENT_MODEL` |
| Config Key | — |

### Type

string (model alias or full ID)

### Default

Binary default (inherits parent model unless overridden)

### Since

v2.1.153 (2026-05-28)

### Description

Overrides the model selection for subagent sessions launched by the `Agent` tool
in multi-agent workflows. Allows the orchestrator session to use a powerful model
(e.g. Opus) while routing routine subagent work to a faster/cheaper model
(e.g. Sonnet or Haiku).

Accepts the same model alias or full model ID format as `--model`.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [042_model.md](042_model.md) | Model selection for primary session |
| doc | [../tool/007_agent.md](../tool/007_agent.md) | Agent tool that spawns subagents |
