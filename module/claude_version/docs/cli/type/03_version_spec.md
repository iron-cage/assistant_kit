# Type :: 3. `VersionSpec`

-- **Summary:** Identifies a Claude Code release target; accepts named aliases or semver strings.
-- **Base Type:** String
-- **Valid Values:** `stable`, `month`, `latest`, or semver (e.g., `1.2.3`)
-- **Default:** `stable`
-- **Used By:** `version::`

Semver is validated by dot-count and digit check; 4-part versions and leading
zeros are rejected.

- **Base type:** String
- **Valid values:** `stable`, `month`, `latest`, or valid semver (e.g., `1.2.3`)
- **Default:** `stable`
- **Validation:** checked against named alias list; semver validated by dot-count and digit check
- **Validation errors:** `"unknown version '{raw}': expected 'stable', 'month', 'latest', or semver like '1.2.3'"`

**Named Aliases:**

| Alias | Resolution |
|-------|-----------|
| `stable` | Pinned stable release (2.1.78) |
| `month`  | ~1 month old release for stability (2.1.74) |
| `latest` | Latest available release |

```sh
clv .version.install version::stable
clv .version.install version::1.2.3
clv .version.install version::1.2.3.4  # error: 4-part rejected
```

### Referenced Parameters

| # | Parameter |
|---|-----------|
| 1 | [`version::`](../param/01_version.md) |
