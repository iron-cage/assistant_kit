# Type Tests

### Scope

- **Purpose**: Document validation edge cases for clr semantic types.
- **Responsibility**: Index of per-type validation test files covering type-level parsing and constraint enforcement.
- **In Scope**: All 12 clr semantic types: `MessageText`, `DirectoryPath`, `TokenLimit`, `ModelName`, `VerbosityLevel`, `SystemPromptText`, `EffortLevel`, `CredentialsFilePath`, `TimeoutSecs`, `JsonSchemaText`, `McpConfigPath`, `FilePath`.
- **Out of Scope**: Command-level tests (→ `command/`), per-parameter tests (→ `param/`).

Per-type validation test indices for `clr`. See [type/readme.md](../../../../docs/cli/type/readme.md) for specification.

### Responsibility Table

| Name | Purpose | Status |
|------|---------|--------|
| `01_message_text.md` | Validation tests for `MessageText` type | ✅ |
| `02_directory_path.md` | Validation tests for `DirectoryPath` type | ✅ |
| `03_token_limit.md` | Validation tests for `TokenLimit` type | ✅ |
| `04_model_name.md` | Validation tests for `ModelName` type | ✅ |
| `05_verbosity_level.md` | Validation tests for `VerbosityLevel` type | ✅ |
| `06_system_prompt_text.md` | Validation tests for `SystemPromptText` type | ✅ |
| `07_effort_level.md` | Validation tests for `EffortLevel` type | ✅ |
| `08_credentials_file_path.md` | Validation tests for `CredentialsFilePath` type | ✅ |
| `09_timeout_secs.md` | Validation tests for `TimeoutSecs` type | ✅ |
| `10_json_schema_text.md` | Validation tests for `JsonSchemaText` type | ✅ |
| `11_mcp_config_path.md` | Validation tests for `McpConfigPath` type | ✅ |
| `12_file_path.md` | Validation tests for `FilePath` type | ✅ |
