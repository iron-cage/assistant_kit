# Type: Session

### Scope

- **Purpose**: Define Session — the rolling usage window with per-session settings overrides.
- **Responsibility**: Documents session identity, window nature, and the settings it may override.
- **In Scope**: Window semantics, per-session overrides, relationship to quota.
- **Out of Scope**: Window lifecycle transitions (→ [state_machine/003](../state_machine/003_session_window_lifecycle.md)); model override resolution order (→ [algorithm/002](../algorithm/002_session_model_override.md)); touch invocation (→ [subprocess/004](../subprocess/004_session_touch_invocation.md)).

### Definition

A rolling five-hour usage window on one account, opened by first use ("touch") and expiring on schedule — the unit the provider meters short-window quota against. Identity is the owning account plus the window's start; state is mutable while the window is open — an entity.

A session may carry settings overrides (model, effort) that apply for its duration and resolve ahead of persistent preferences ([algorithm/002](../algorithm/002_session_model_override.md)).

### Validation

- At most one open window per account at a time; a touch inside an open window never opens a second one.
- Window boundaries derive from provider-reported reset times, not local arithmetic, where available.
- Override keys are the closed set governed by the session-settings surface — unknown keys rejected.

### Relationships

Windows are what [Quota Snapshot (008)](008_quota_snapshot.md)'s five-hour measurements describe; session touch behavior in [subprocess/004](../subprocess/004_session_touch_invocation.md); lifecycle in [state_machine/003](../state_machine/003_session_window_lifecycle.md).

### Serialization

Session-scoped settings live in `~/.claude/settings.json` ([schema/006](../schema/006_settings_json.md)); window state is provider-side, observed via usage responses.
