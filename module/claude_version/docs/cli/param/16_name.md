# Parameter :: 16. `name::`

-- **Summary:** Marker name for `.version.mark` CRUD operations (required).
-- **Type:** string
-- **Default:** — (required; no default)
-- **Commands:** `.version.mark`
-- **Group:** —

The name of the custom marker to create, update, or remove. Must satisfy `[a-z][a-z0-9-]*` (lowercase letter start, followed by lowercase letters, digits, or hyphens), be at most 32 characters, and must not collide with any built-in alias (`stable`, `latest`).

- **Type:** string
- **Default:** none — `name::` is required on every `.version.mark` invocation; its absence causes exit 1
- **Validation:**
  - Must match `[a-z][a-z0-9-]*`
  - Maximum 32 characters
  - Must not shadow a built-in alias (`stable` or `latest`)

```sh
clv .version.mark name::team-pin version::2.1.220   # create or update marker
clv .version.mark name::team-pin unset::1            # remove marker
clv .version.mark name::team-pin version::2.1.220 dry::1  # preview without writing
```

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`.version.mark`](../command/version.md#command-17-versionmark) | — (required) | Required on every invocation; controls which marker entry is read or written |

### Referenced Type

| # | Type |
|---|------|
| 1 | `string` |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [005 Version Pinning](../user_story/005_version_pinning.md) | Developer (version pinning) |
