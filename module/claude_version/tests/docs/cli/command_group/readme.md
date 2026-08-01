# Command Group Tests

### Scope

- **Purpose**: Document structural-equivalence tests for clv command groups.
- **Responsibility**: Index of per-command-group test files verifying shared-routine and shared-parameter-set claims.
- **In Scope**: All clv command group test files (0 groups currently qualify).
- **Out of Scope**: Per-command tests (-> `command/`), per-parameter-group interaction tests (-> `param_group/`).

Per-group structural-equivalence test indices for `clv`. See [command_group/readme.md](../../../../docs/cli/command_group/readme.md) for specification.

### Overview Table

| Name | Purpose | Status |
|------|---------|--------|
| *(none)* | 0 command groups qualify under the strict identity test — see [`command_group/readme.md`](../../../../docs/cli/command_group/readme.md) Evaluated, Not Qualifying for the 3 candidate pairs evaluated and rejected | N/A |
| procedure.md | Workflow for creating and updating command group test specs | ✅ |

**Total:** 0 test files. No `command_group` in `claude_version` has an identical routine function and identical parameter set (differing at most by default), so there is nothing to structurally verify at this tier yet. If a future refactor introduces a genuine alias (one command delegating its full dispatch to another's routine, as claude_runner's `ask -> run` does), add its test spec here following `procedure.md`.
