# Decision: Duplicate Flags Last Wins

**ID:** D6 · **Category:** Parameter Conventions · **Status:** ✅ Adopted

### Scope

- **Purpose**: Record why a repeated value-flag resolves to its last occurrence rather than erroring or accumulating.
- **Responsibility**: Rationale for last-wins resolution and the wrapper-script use case it enables.
- **In Scope**: The resolution rule for duplicate value-flags and the convention it matches.
- **Out of Scope**: Precedence between different configuration *sources* — CLI arg vs env var vs JSON config (→ [`../feature/004_json_config.md`](../feature/004_json_config.md)).

### Decision

When a flag like `--model` appears twice, the last value wins.

### Rationale

Matches curl/git convention, so the behavior is already familiar. More importantly it enables wrapper scripts to override defaults: a script can emit its own `--model` and then append `"$@"`, and any `--model` the caller passes lands later on the command line and therefore takes effect. Erroring on duplicates would make that pattern impossible without the wrapper parsing its own arguments first.

### Consequence

`clr --model opus --model sonnet` runs `sonnet`. A wrapper's baked-in defaults are always overridable by the caller without the wrapper needing to know which flags it set.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Decision collection index |
| feature | [`../feature/004_json_config.md`](../feature/004_json_config.md) | Precedence across configuration sources, layered above this within-command-line rule |
| test | `../../tests/cli_args_test.rs` | Last-wins duplicate resolution coverage |
