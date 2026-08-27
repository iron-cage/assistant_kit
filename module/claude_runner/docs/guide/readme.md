# guide/

### Scope

**Responsibilities:** Narrative, reproducible operational walkthroughs for the `claude_runner` crate — each making one concrete multi-step task repeatable by someone who was not present when it was first worked out.
**In Scope:** Operational, setup, and migration walkthroughs; prerequisites with verification sources; phased runbook steps with State-Check Sandwich verification; open decisions left explicitly unresolved.
**Out of Scope:** Behavioral contracts (→ `../feature/`, `../invariant/`, `../api/`), CLI reference (→ `../cli/`), tracked work items (→ the workspace `task/` tree), prescriptive rule sets (→ `*.rulebook.md`).

Governed by `$GENAI/dev/doc/guide_des.rulebook.md`.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| [001_topic_sessions.md](001_topic_sessions.md) | Forking the current session into isolated, resumable topic sessions |
| [002_hosted_sessions.md](002_hosted_sessions.md) | Holding one interactive conversation open across separate commands |

### Guides

| # | Guide | Purpose |
|---|-------|---------|
| 1 | [001_topic_sessions.md](001_topic_sessions.md) | Fork the current conversation into one or more isolated topic sessions, locally or in the global topic home, and return to any of them by name |
| 2 | [002_hosted_sessions.md](002_hosted_sessions.md) | Hold one real interactive session open across many separate `clr chat` commands — print-mode shape, with the conversation surviving between calls — and tear it down when done |
