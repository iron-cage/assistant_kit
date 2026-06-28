# Output Formats

### Scope

- **Purpose**: Define the named output format catalog for the clv CLI.
- **Responsibility**: One file per named format type with structure and example.
- **In Scope**: text (human-readable) and json (machine-readable) output formats.
- **Out of Scope**: Command-specific output (-> `../command/`), behavioral rules (-> `../../feature/`).

### Responsibility Table

| File | Responsibility |
|------|---------------|
| readme.md | Index, scope, and Overview Table for format instances |
| procedure.md | Steps for adding, updating, or removing format instances |
| 001_text.md | Human-readable labeled text output format |
| 002_json.md | Machine-readable structured JSON output format |

### Overview Table

| File | Format | Purpose |
|------|--------|---------|
| [001_text.md](001_text.md) | `text` | Human-readable labeled output |
| [002_json.md](002_json.md) | `json` | Machine-readable structured output |
