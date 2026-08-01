# Parameter: 73. `inference_provider::` (metadata label)

Specifies the inference provider label to store in the account profile at `.account.save` time. Displayed by default (identity default set) in `.accounts`; governs Gate 10 rotation grouping.

- **Default:** *(omit; field left absent in `{name}.json` — readers treat absence as `"anthropic"`)*
- **Constraints:** Any non-empty string; no format validation, no allow-list — free-form, mirroring `host::`/`role::`
- **Purpose:** Tag a saved account with the inference provider it authenticates against (e.g., `anthropic`, `kimi`, `moonshot`). Used exclusively for Gate 10 rotation grouping — never as a filter.

**Behavior:** The value is written to the `inference_provider` field in `{name}.json` via read-merge, alongside `host::`/`role::`. It persists until `.account.save` is re-run with a different `inference_provider::` value. When omitted, the field is left absent (not written as the literal string `"anthropic"`) — both `list()` (`.accounts`/`.usage` rendering) and Gate 10 (rotation eligibility) treat an absent field as `"anthropic"`. An empty `inference_provider::` value is a usage error (exit 1), unlike `host::`/`role::` which accept empty strings.

**Examples:**

```text
clp .account.save                                        -> inference_provider left absent (reads as "anthropic")
clp .account.save inference_provider::kimi                -> inference_provider stored as "kimi"
clp .account.save host::workstation inference_provider::moonshot
                                                            -> host "workstation", inference_provider "moonshot"
clp .account.save inference_provider::                     -> exit 1: inference_provider:: must be non-empty
```

**See Also:** [feature/072_inference_provider_selection.md](../../feature/072_inference_provider_selection.md) for the full provider-selection feature. [009_provider.md](../command/009_provider.md) for the `.provider.select` global config command that Gate 10 compares this field against.

### Referenced Type

- **Fundamental Type:** `string`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Account Targeting](../param_group/006_account_targeting.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.save`](../command/001_account.md#command-4-accountsave) | Write inference provider metadata label to account profile |

### Referenced Algorithms

| # | Algorithm | Role |
|---|-----------|------|
| 1 | [Eligibility Gates](../../algorithm/004_eligibility_gates.md) | Gate 10 — compares this field against the selected `provider` config value |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Tagging inference provider during account profile creation |
