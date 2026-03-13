# Types

### All Types (5 total)

| # | Type | Base | Used By | Purpose |
|---|------|------|---------|---------|
| 1 | `VerbosityLevel` | u8 (0–2) | [`v::`](params.md#parameter--4-v) | Output detail control |
| 2 | `OutputFormat` | enum | [`format::`](params.md#parameter--5-format) | Display format selection |
| 3 | `VersionSpec` | String | [`version::`](params.md#parameter--1-version) | Release target identifier |
| 4 | `SettingsKey` | String | [`key::`](params.md#parameter--6-key) | Settings entry name |
| 5 | `SettingsValue` | String | [`value::`](params.md#parameter--7-value) | Settings entry value (type-inferred) |

---

### Type :: 1. `VerbosityLevel`

Controls the detail level of command output. Range 0–2 with different
semantics from claude_runner's 0–5 range.

- **Base type:** u8
- **Constraints:** 0 to 2
- **Default:** 1 (normal)
- **Parsing:** `str::parse::<u8>()`; rejects non-integer and out-of-range
- **Validation errors:**
  - Non-integer: `"verbosity must be 0, 1, or 2, got: '{raw}'"`
  - Out of range: `"verbosity out of range: {n} (max 2)"`
- **Used by:** [`v::`](params.md#parameter--4-v)

**Level Semantics:**

| Level | Name | Output |
|-------|------|--------|
| 0 | minimal | Raw values only (no labels) |
| 1 | normal | Labeled key-value pairs (default) |
| 2 | verbose | Diagnostic details, extra context |

```sh
cm .status v::0       # minimal
cm .status v::2       # verbose
cm .status v::3       # error: out of range
```

---

### Type :: 2. `OutputFormat`

Select between human-readable text and machine-readable JSON output.
Case-sensitive matching.

- **Base type:** enum (2 variants)
- **Valid values:** `text`, `json`
- **Default:** `text`
- **Parsing:** exact string match; `Text`, `JSON`, `Json` all rejected
- **Validation errors:** `"unknown format '{raw}': expected text or json"`
- **Used by:** [`format::`](params.md#parameter--5-format)

```sh
cm .status format::text       # human-readable
cm .status format::json       # machine-readable
cm .status format::JSON       # error: case-sensitive
```

---

### Type :: 3. `VersionSpec`

Identifies a Claude Code release target. Accepts named aliases or
semver strings.

- **Base type:** String
- **Valid values:** `stable`, `month`, `latest`, or valid semver (e.g., `1.2.3`)
- **Default:** `stable`
- **Parsing:** checked against `VERSION_ALIASES` map; semver validated by dot-split digit check
- **Validation errors:** `"unknown version '{raw}': expected 'stable', 'month', 'latest', or semver like '1.2.3'"`
- **Used by:** [`version::`](params.md#parameter--1-version)

**Named Aliases:**

| Alias | Resolution |
|-------|-----------|
| `stable` | Pinned stable release (2.1.78) |
| `month`  | ~1 month old release for stability (2.1.74) |
| `latest` | Latest available release |

```sh
cm .version.install version::stable
cm .version.install version::1.2.3
cm .version.install version::1.2.3.4  # error: 4-part rejected
```

---

### Type :: 4. `SettingsKey`

Name of a settings entry in `~/.claude/settings.json`. Stored as a
literal JSON object key (dot characters are literal, not path separators).

- **Base type:** String
- **Constraints:** non-empty; any UTF-8 string
- **Validation:** `"key:: is required"` if missing; `"key:: value cannot be empty"` if empty
- **Used by:** [`key::`](params.md#parameter--6-key)

```sh
cm .settings.get key::theme
cm .settings.get key::api.endpoint   # dot is literal
```

---

### Type :: 5. `SettingsValue`

Value to write to a settings entry. Automatically type-inferred for JSON
storage using `infer_type()` in `settings_io.rs`.

- **Base type:** String (auto-typed for JSON serialization)
- **Constraints:** non-empty; any UTF-8 string
- **Validation:** `"value:: is required"` if missing; `"value:: value cannot be empty"` if empty
- **Used by:** [`value::`](params.md#parameter--7-value)

**Type Inference Rules:**

| Input | Inferred Type | JSON Output |
|-------|---------------|-------------|
| `"true"` / `"false"` | Bool | `true` / `false` |
| Integer string (e.g., `"42"`) | Number (i64) | `42` |
| Finite float string (e.g., `"3.14"`) | Number (f64) | `3.14` |
| `"NaN"`, `"inf"`, `"infinity"` | String | `"NaN"`, `"inf"` |
| Everything else | String | `"value"` |

**Note:** Non-finite floats (`NaN`, `inf`, `infinity` and variants) are
classified as strings because they are not valid JSON number literals.
Special characters (`"`, `\`) in string values are properly escaped.

```sh
cm .settings.set key::autoUpdate value::true    # -> true (bool)
cm .settings.set key::timeout value::30         # -> 30 (number)
cm .settings.set key::theme value::dark         # -> "dark" (string)
cm .settings.set key::rate value::3.14          # -> 3.14 (number)
cm .settings.set key::special value::NaN        # -> "NaN" (string)
```


