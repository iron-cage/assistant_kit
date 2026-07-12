# API: `CanUseTool` Permission Callback

### Scope

- **Purpose**: Document the per-tool-call permission override callback, distinct from the session-wide `permissionMode` setting.
- **Responsibility**: Authoritative signature reference for `CanUseTool`.
- **In Scope**: The callback signature and its parameters.
- **Out of Scope**: `permissionMode` itself (→ [`../param/003_permission_mode.md`](../param/003_permission_mode.md)); the mode enum's values (→ [`../behavior/006_s6_permission_modes_richer_than_cli.md`](../behavior/006_s6_permission_modes_richer_than_cli.md)).

```typescript
type CanUseTool = (
  toolName: string,
  input: Record<string, unknown>,
  options: {
    signal: AbortSignal;
    suggestions?: PermissionUpdate[];
    blockedPath?: string;
    decisionReason?: string;
    toolUseID: string;
    agentID?: string;
    requestId: string;
  }
) => Promise<PermissionResult | null>;
```

Set via `options.canUseTool` on `query()`. Unlike `permissionMode` (one session-wide posture) or `setPermissionMode()` (one session-wide posture, changeable mid-session — see [`004_query_control_object.md`](004_query_control_object.md)), `canUseTool` is invoked once **per tool call**, receiving the specific `toolName` and `input` being requested, and can return a per-call `PermissionResult` (allow/deny/modify — exact `PermissionResult` shape not fetched in this pass) or `null` to fall through to the session's `permissionMode` default. The presence of `agentID` in the callback's third argument confirms this fires for subagent-originated tool calls too, not just the top-level session.

This is the most direct SDK analogue of a Rust bridge's own authorization layer: a Rust-native SDK-protocol integration wanting fine-grained, per-call tool gating (rather than the coarser `allowedTools`/`disallowedTools` static lists) would need its bridge process to implement the equivalent of this callback against the control protocol directly, since `canUseTool` itself is JS/Python-only.

### Params

| File | Relationship |
|------|--------------|
| [../param/003_permission_mode.md](../param/003_permission_mode.md) | The coarser, session-wide setting this callback overrides per call |
| [../param/004_allowed_tools.md](../param/004_allowed_tools.md) | The static-list alternative to this dynamic callback |

### Behaviors

| File | Relationship |
|------|--------------|
| [../behavior/006_s6_permission_modes_richer_than_cli.md](../behavior/006_s6_permission_modes_richer_than_cli.md) | Session-wide mode this callback layers on top of |
