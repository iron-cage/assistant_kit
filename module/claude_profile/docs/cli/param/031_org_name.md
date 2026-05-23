# Parameter: `org_name::`

**Type:** `bool`
**Default:** `0` (off)
**Valid values:** `0`, `1`, `false`, `true`
**Commands:** `.credentials.status`, `.accounts`
**Group:** [Field Presence](../param_group/002_field_presence.md)

Show the organization display name from `{name}.roles.json` (populated at `save()` time via endpoint 005).

## Behaviour

When `org_name::1`, appends an `Org:` line showing the account's `organization_name` value (e.g. `"alice@example.com's Organization"`). Sources from `{name}.roles.json` in the credential store. Shows `N/A` when `{name}.roles.json` is absent or the field is missing.

For `.credentials.status`: reads from the active account's `{_active}.roles.json`; `N/A` when no active account or no roles snapshot.

`format::json` always includes `organization_name` regardless of this param.

## Output

```
Org: alice@example.com's Organization
```

## Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [../../feature/022_org_identity_snapshot.md](../../feature/022_org_identity_snapshot.md) | Org identity snapshot feature spec |
| doc | [002_field_presence.md](../param_group/002_field_presence.md) | Field presence parameter group |
