# Parameter: `org_uuid::`

**Type:** `bool`
**Default:** `0` (off)
**Valid values:** `0`, `1`, `false`, `true`
**Commands:** `.credentials.status`, `.accounts`
**Group:** [Field Presence](../param_group/002_field_presence.md)

Show the organization UUID from `{name}.json` (populated at `save()` time via endpoint 005).

## Behaviour

When `org_uuid::1`, appends an `Org ID:` line showing the account's `organization_uuid` value (a UUID string). Sources from `{name}.json` in the credential store. Shows `N/A` when `{name}.json` is absent or the field is missing.

For `.credentials.status`: reads from the active account's `{active_account}.json`; `N/A` when no active account or no roles snapshot.

`format::json` always includes `organization_uuid` regardless of this param.

## Output

```
Org ID: aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee
```

## Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [../../feature/022_org_identity_snapshot.md](../../feature/022_org_identity_snapshot.md) | Org identity snapshot feature spec |
| doc | [002_field_presence.md](../param_group/002_field_presence.md) | Field presence parameter group |
