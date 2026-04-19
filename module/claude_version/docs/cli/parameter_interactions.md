# Parameter Interactions

How cm parameters interact when combined. See [params.md](params.md) and [parameter_groups.md](parameter_groups.md) for individual parameter specifications.

## Interaction Table

| Parameters | Interaction | Outcome |
|------------|-------------|---------|
| `dry::1` + `force::1` | Precedence | `dry::` wins; action previewed, not executed |
| `dry::1` + `force::0` | Independent | Dry-run preview shown; `force::` has no effect |
| `dry::0` + `force::1` | Independent | Confirmation skipped; command executes |
| `key::` + `value::` | Required pair | Both required for `.settings.set`; either alone is invalid |
| `v::0` + `format::json` | Independent | JSON output rendered at minimal verbosity; required keys not stripped |
| `v::2` + `format::text` | Independent | Text output rendered with extra diagnostic lines |
| `version::` + `force::1` | Additive | Installs specified version; skips "already installed" guard |
| `version::` + `dry::1` | Additive | Shows install plan for specified version; no install occurs |

## Required Combinations

`.settings.set` requires both `key::` and `value::`:

```
.settings.set key::theme value::dark   -- both present: valid
.settings.set key::theme               -- value:: missing: exit 1
.settings.set value::dark              -- key:: missing: exit 1
```

## Precedence Rules

When `dry::1` and `force::1` are both present, `dry::` takes precedence:

```
.version.install version::stable dry::1 force::1
  -- Output: dry-run preview of install
  -- Action: no install executed (dry wins)
```

## Independent Parameters

`v::` and `format::` operate on orthogonal dimensions (depth vs structure) and do not interact:

- `v::0` controls how many fields are shown
- `format::json` controls the structure of those fields
- Combining them applies both transformations independently

## Scope

`version::` applies to version commands: `.version.install`, `.version.guard` (override-only on guard; stored preference unchanged).

`dry::` applies to mutation commands: `.version.install`, `.version.guard`, `.processes.kill`, `.settings.set`.

`force::` applies to mutation commands with safety guards: `.version.install`, `.version.guard`, `.processes.kill`.

`v::` applies to output-capable commands: `.status`, `.version.show`, `.version.install`, `.version.list`, `.version.guard`, `.version.history`, `.processes`, `.processes.kill`, `.settings.show`, `.settings.get`.

`format::` applies to output-capable commands: `.status`, `.version.show`, `.version.install`, `.version.list`, `.version.guard`, `.version.history`, `.processes`, `.processes.kill`, `.settings.show`, `.settings.get`.
