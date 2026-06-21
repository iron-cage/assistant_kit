# Parameter: disable_prompt_caching

### Forms

| Form | Value |
|------|-------|
| Env Var | `DISABLE_PROMPT_CACHING` |

### Type

boolean (presence-activated)

### Default

Not set (prompt caching enabled)

### Description

Disables prompt caching for ALL models. For per-model control, use the
model-specific variants: `DISABLE_PROMPT_CACHING_FABLE`,
`DISABLE_PROMPT_CACHING_HAIKU`, `DISABLE_PROMPT_CACHING_OPUS`,
`DISABLE_PROMPT_CACHING_SONNET`.

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [114_disable_prompt_caching_fable.md](114_disable_prompt_caching_fable.md) | Fable-specific |
| doc | [115_disable_prompt_caching_haiku.md](115_disable_prompt_caching_haiku.md) | Haiku-specific |
| doc | [116_disable_prompt_caching_opus.md](116_disable_prompt_caching_opus.md) | Opus-specific |
| doc | [117_disable_prompt_caching_sonnet.md](117_disable_prompt_caching_sonnet.md) | Sonnet-specific |
