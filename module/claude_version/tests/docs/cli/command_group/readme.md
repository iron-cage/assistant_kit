# Command Group Tests

### Scope

- **Purpose**: Document structural-equivalence tests for clv command groups.
- **Responsibility**: Index of per-command-group test files verifying shared-routine and shared-parameter-set claims.
- **In Scope**: All clv command group test files (0 multi-member groups currently qualify; all 16 documented groups are Singleton Groups, which need no structural-equivalence test).
- **Out of Scope**: Per-command tests (-> `command/`), per-parameter-group interaction tests (-> `param_group/`).

Per-group structural-equivalence test indices for `clv`. See [command_group/readme.md](../../../../docs/cli/command_group/readme.md) for specification.

### Overview Table

| Name | Purpose | Status |
|------|---------|--------|
| *(none)* | 0 multi-member command groups qualify under the strict identity test — all 16 groups in `command_group/readme.md`'s All Groups table are Singleton Groups, which have no sibling to prove equivalence against; see [`command_group/readme.md`](../../../../docs/cli/command_group/readme.md) Evaluated, Not Qualifying for the 6 candidate pairs evaluated and rejected | N/A |
| procedure.md | Workflow for creating and updating command group test specs | ✅ |

**Total:** 0 test files. No `command_group` in `claude_version` has an identical routine function and identical parameter set (differing at most by default), so there is nothing to structurally verify at this tier yet — Singleton Groups are exempt from this requirement by definition. If a future refactor introduces a genuine alias (one command delegating its full dispatch to another's routine, as claude_runner's `ask -> run` does), add its test spec here following `procedure.md`.
