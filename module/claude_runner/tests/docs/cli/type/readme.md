# Type Tests

### Scope

- **Purpose**: Document validation edge cases for clr semantic types.
- **Responsibility**: Index of per-type validation test files covering type-level parsing and constraint enforcement.
- **In Scope**: All 11 clr semantic types.
- **Out of Scope**: Command-level tests (→ `command/`), per-parameter tests (→ `param/`).

Per-type validation test indices for `clr`. See [type.md](../../../../docs/cli/type.md) for specification.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| 01_message_text.md | Validation tests for `MessageText` type |
| 02_directory_path.md | Validation tests for `DirectoryPath` type |
| 03_token_limit.md | Validation tests for `TokenLimit` type |
| 04_model_name.md | Validation tests for `ModelName` type |
| 05_verbosity_level.md | Validation tests for `VerbosityLevel` type |
| 06_system_prompt_text.md | Validation tests for `SystemPromptText` type |
| 07_effort_level.md | Validation tests for `EffortLevel` type |
| 08_credentials_file_path.md | Validation tests for `CredentialsFilePath` type |
| 09_timeout_secs.md | Validation tests for `TimeoutSecs` type |
| 10_json_schema_text.md | Validation tests for `JsonSchemaText` type |
| 11_mcp_config_path.md | Validation tests for `McpConfigPath` type |

### Index

| Type | File | Tests |
|------|------|-------|
| `MessageText` | [01_message_text.md](01_message_text.md) | 5 TC |
| `DirectoryPath` | [02_directory_path.md](02_directory_path.md) | 4 TC |
| `TokenLimit` | [03_token_limit.md](03_token_limit.md) | 6 TC |
| `ModelName` | [04_model_name.md](04_model_name.md) | 4 TC |
| `VerbosityLevel` | [05_verbosity_level.md](05_verbosity_level.md) | 5 TC |
| `SystemPromptText` | [06_system_prompt_text.md](06_system_prompt_text.md) | 4 TC |
| `EffortLevel` | [07_effort_level.md](07_effort_level.md) | 6 TC |
| `CredentialsFilePath` | [08_credentials_file_path.md](08_credentials_file_path.md) | 6 TC |
| `TimeoutSecs` | [09_timeout_secs.md](09_timeout_secs.md) | 6 TC |
| `JsonSchemaText` | [10_json_schema_text.md](10_json_schema_text.md) | 4 TC |
| `McpConfigPath` | [11_mcp_config_path.md](11_mcp_config_path.md) | 4 TC |
