# Feature Doc Entity

### Scope

- **Purpose**: Document the capabilities of the `claude_topic_core` library for consumers that need to name, find, choose among, or exclusively hold a topic.
- **Responsibility**: Index of feature doc instances covering identity, enumeration, selection, pooling, and locking.
- **In Scope**: The `( name, mode )` pair and its resolution; merging two mechanisms into one listing; idle-first drawing; idempotent pool naming; advisory per-topic exclusion.
- **Out of Scope**: Invoking Claude Code for a topic — creating, forking, transplanting, or resuming (→ `claude_runner/docs/cli/command/`); session path computation (→ `claude_storage_core/docs/`); invariant constraints (→ `invariant/`); signature contracts (→ `api/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Topic Identity](001_topic_identity.md) | What a `--topic` name resolves to, and which of the two mechanisms answers | ✅ |
| 002 | [Topic Enumeration](002_topic_enumeration.md) | Which topics exist under a base, both mechanisms merged | ✅ |
| 003 | [Topic Selection](003_topic_selection.md) | Which topic a forwarded prompt should go to | ✅ |
| 004 | [Topic Pool](004_topic_pool.md) | Naming N anonymous topics idempotently | ✅ |
| 005 | [Topic Lock](005_topic_lock.md) | Keeping two writers off one conversation | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating feature doc instances | ✅ |
