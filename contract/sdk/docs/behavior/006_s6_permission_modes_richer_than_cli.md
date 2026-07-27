# Behavior S6: SDK Permission Modes Are a Richer Enum Than the CLI Surface

### Scope

- **Purpose**: Document the SDK's 6-member `PermissionMode` enum and flag it as richer than the CLI's `--permission-mode` flag already catalogued in `contract/claude_code`.
- **Responsibility**: Authoritative instance for behavior S6.
- **In Scope**: The `PermissionMode` union's 6 values and their one-line semantics; the existence of a live `setPermissionMode()` mid-session control method with no CLI equivalent.
- **Out of Scope**: The CLI-level `--permission-mode` flag itself (→ [`../../../claude_code/docs/param/046_permission_mode.md`](../../../claude_code/docs/param/046_permission_mode.md)); the `canUseTool` callback that can override any mode's default decision (→ [S-none — see `../api/006_permission_callback.md`](../api/006_permission_callback.md) directly, no dedicated behavior instance).

### Behavior

**Status**: 🎯 Observed | **Certainty**: 80% | **Since**: SDK GA | **Evidence**: E2, E7

The TypeScript reference documents `PermissionMode` as a 6-member string union: `"default"` (standard behavior), `"acceptEdits"` (auto-accept file edits), `"bypassPermissions"` (bypass permission checks — "explicit ask rules still prompt", a caveat the CLI's own `--dangerously-skip-permissions` flag does not carry), `"plan"` (explore without editing), `"dontAsk"` (deny anything not pre-approved instead of prompting), and `"auto"` (delegate the approve/deny decision to a model classifier). Two of these — `"dontAsk"` and `"auto"` — have no obviously equivalent single CLI flag documented in `contract/claude_code/docs/param/`; they read as SDK-native additions layered on top of the CLI's simpler permission surface, not a re-export of it.

Additionally, the `Query` control object exposes `setPermissionMode(mode): Promise<void>` as a live, mid-session method — the running agent's permission posture can change *after* `query()` has already started, which has no CLI analogue at all (the CLI's `--permission-mode` is argv-time only, fixed for the process's lifetime).

**Certainty caveat**: marked 🎯 Observed rather than ✅ Confirmed because this instance asserts a *comparison* ("richer than the CLI surface") rather than a single documented fact — confirming it fully would require enumerating every CLI permission-related flag/setting in `contract/claude_code/docs/param/` and checking each `PermissionMode` value maps to zero, one, or many of them; that cross-check has not been performed line-by-line here.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E2 | S6 | Doc | `https://code.claude.com/docs/en/agent-sdk/typescript` | `Query` interface | `setPermissionMode(mode: PermissionMode): Promise<void>` — live mid-session control, no CLI equivalent |
| E7 | S6 | Doc | `https://code.claude.com/docs/en/agent-sdk/typescript` | `PermissionMode` type definition | 6-member union with inline semantics for each value, verbatim |

### Cross-References

| Type | File | Responsibility |
|------|------|-----------------|
| entity | [readme.md](readme.md) | Master index |
| param | [../param/003_permission_mode.md](../param/003_permission_mode.md) | `permissionMode` field detail |
| api | [../api/004_query_control_object.md](../api/004_query_control_object.md) | `setPermissionMode()` control method |
| api | [../api/006_permission_callback.md](../api/006_permission_callback.md) | `canUseTool` callback — a further, per-call override layer on top of the mode |
| doc | [`../../../claude_code/docs/param/046_permission_mode.md`](../../../claude_code/docs/param/046_permission_mode.md) | The narrower CLI-level flag this compares against |
