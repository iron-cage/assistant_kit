# Parameter: `uuid::`

**Type:** `bool`
**Default:** `0` (off)
**Valid values:** `0`, `1`, `false`, `true`
**Commands:** `.credentials.status`, `.accounts`
**Group:** [Field Presence](../param_group/002_field_presence.md)

Show the stable user identifier (`taggedId`) from the `oauthAccount` object in `{name}.json`.

## Behaviour

When `uuid::1`, appends an `ID:` line showing the account's `taggedId` value (e.g. `"user_01..."`) sourced from the `{name}.json` snapshot. Shows `N/A` when the snapshot is absent or the field is missing.

`format::json` always includes `tagged_id` regardless of this param.

## Output

```
ID:  user_01ABCDEFGhijklmnopqrstuvwx
```

## Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [../../feature/021_extended_snapshot_fields.md](../../feature/021_extended_snapshot_fields.md) | Feature spec for `uuid::` and `capabilities::` |
| doc | [002_field_presence.md](../param_group/002_field_presence.md) | Field presence parameter group |
