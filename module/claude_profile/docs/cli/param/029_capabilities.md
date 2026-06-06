# Parameter: `capabilities::`

**Type:** `bool`
**Default:** `0` (off)
**Valid values:** `0`, `1`, `false`, `true`
**Commands:** `.credentials.status`, `.accounts`
**Group:** [Field Presence](../param_group/002_field_presence.md)

Show the enabled product feature list (`capabilities`) from the `oauthAccount` object in `{name}.json`.

## Behaviour

When `capabilities::1`, appends a `Capabilities:` line showing the account's capabilities as a comma-separated list (e.g. `max, chat`). Sources from the `capabilities` string array inside `oauthAccount` in the `{name}.json` snapshot. Shows `N/A` when the snapshot is absent, the field is missing, or the array is empty.

`format::json` always includes `capabilities` as a JSON array regardless of this param.

## Output

```
Capabilities: max, chat
```

## Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [../../feature/021_extended_snapshot_fields.md](../../feature/021_extended_snapshot_fields.md) | Feature spec for `uuid::` and `capabilities::` |
| doc | [002_field_presence.md](../param_group/002_field_presence.md) | Field presence parameter group |
