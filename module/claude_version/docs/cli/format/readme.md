# Output Formats

### Scope

- **Purpose**: Define the named output format catalog for the clv CLI.
- **Responsibility**: One file per named format type with structure and example.
- **In Scope**: text (human-readable) and json (machine-readable) output formats.
- **Out of Scope**: Command-specific output (-> `../command/`), behavioral rules (-> `../../feature/`).

### Overview Table

| File | Format | Purpose |
|------|--------|---------|
| [01_text.md](01_text.md) | `text` | Human-readable labeled output |
| [02_json.md](02_json.md) | `json` | Machine-readable structured output |
