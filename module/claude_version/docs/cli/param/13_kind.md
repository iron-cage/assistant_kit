# Parameter :: 13. `kind::`

-- **Summary:** Filter show-all output to params of one kind.
-- **Type:** `ParamKind`
-- **Default:** absent (all params)
-- **Commands:** `.params`
-- **Group:** none

When absent, `.params` shows all catalog parameters. When set to `config` or `env`, only parameters with the matching form type are shown. Has no effect when `key::` is provided (single-param mode supersedes this filter).

- **Type:** [`ParamKind`](../type/08_param_kind.md)
- **Default:** absent (no filter — all params)
- **Validation:** must be `config` or `env`; any other value → exit 1
- **Mode constraint:** ignored when `key::` is also provided (single-param mode takes precedence)

```sh
clv.params kind::config     # show only settings.json config params
clv.params kind::env        # show only env var params with current env values
clv.params                  # absent = all params (default)
```

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`.params`](../command/params.md#command-14-params) | absent | Ignored when `key::` is also present (single-param mode) |

### Referenced Type

| # | Type |
|---|------|
| 1 | [`ParamKind`](../type/08_param_kind.md) |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [007 Params Inspection](../user_story/007_params_inspection.md) | Developer (config inspector) |
