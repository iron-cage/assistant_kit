# API Doc Entity

### Scope

- **Purpose**: Document the programmatic interface of the claude_core library surface.
- **Responsibility**: Index of API doc instances covering the `settings_io` and `toml_io` contracts.
- **In Scope**: `settings_io` atomic JSON key-value read/write functions and the `StoredAs` type; `toml_io` tiered flat-TOML key-value read/write functions.
- **Out of Scope**: Path/process primitives (undocumented at this level — no doc instance exists for `paths`/`process` yet), CLI binary behavior (this crate has no binary).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Settings I/O](001_settings_io.md) | Atomic flat-JSON KV read/write contract | ✅ |
| 002 | [TOML I/O](002_toml_io.md) | Tiered (project + user) flat-TOML KV read/write contract | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating API doc instances | ✅ |
