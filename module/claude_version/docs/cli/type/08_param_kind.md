# Type :: 8. `ParamKind`

-- **Summary:** Select which kind of params to show in `.params` show-all mode.
-- **Base Type:** enum (2 variants)
-- **Valid Values:** `config`, `env`
-- **Default:** absent (no filter)
-- **Used By:** `kind::`

Case-sensitive matching. `Config`, `ENV`, `Env` are all rejected.

- **Base type:** enum (2 variants)
- **Valid values:** `config`, `env`
- **Default:** absent (no filter — all catalog params shown)
- **Parsing:** exact string match; `Config`, `ENV` all rejected
- **Validation errors:** `"unknown kind '{raw}': expected config or env"`

```sh
clv.params kind::config     # settings.json config params only
clv.params kind::env        # env var params only
clv.params kind::Config     # error: case-sensitive
```

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|--------------|
| 1 | [`.params`](../command/params.md#command--14-params) | `kind::` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|---------|
| 1 | [`kind::`](../param/13_kind.md) | 1 |
