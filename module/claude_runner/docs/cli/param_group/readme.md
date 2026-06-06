# Parameter Groups

### Scope

- **Purpose**: Define semantically coherent parameter clusters by runner responsibility.
- **Responsibility**: Group 32 parameters into 4 categories based on consumption pattern (claude-native, runner-consumed, system-prompt, credential-ops).
- **In Scope**: Group membership, coherence tests, invariants, and cross-references to commands/params/tests/user stories.
- **Out of Scope**: Individual parameter semantics (-> `../param/`), type constraints (-> `../type/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| 01_claude_native_flags.md | Group spec: flags forwarded as-is to claude subprocess |
| 02_runner_control.md | Group spec: flags consumed by runner before subprocess launch |
| 03_system_prompt.md | Group spec: system prompt injection and extension flags |
| 04_credential_operations.md | Group spec: credential-isolated execution configuration |

### All Groups (4 total)

| # | Group | Parameters | Purpose |
|---|-------|------------|---------|
| 1 | Claude-Native Flags | 6 | Pass selected `claude` binary flags through without runner modification |
| 2 | Runner Control | 21 | Control runner execution behavior -- before, during, or instead of subprocess invocation |
| 3 | System Prompt | 2 | Inject or extend the behavioral system context sent to the `claude` subprocess |
| 4 | Credential Operations | 3 | Configure credential-isolated execution for `clr isolated` and `clr refresh` |

**Total:** 4 groups
