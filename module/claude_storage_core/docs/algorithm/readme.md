# Algorithm Doc Entity

### Scope

- **Purpose**: Document computational procedures that are non-obvious or have significant design rationale.
- **Responsibility**: Index of algorithm doc instances covering procedure design, tradeoffs, and correctness guarantees.
- **In Scope**: Path encoding/decoding schemes, zero-dependency parser design, and correctness properties.
- **Out of Scope**: In-memory data models (→ `data_structure/`), API surface (→ `api/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Path Encoding](001_path_encoding.md) | v1/v2 filesystem path encoding/decoding scheme | ✅ |
| 002 | [JSON Parser](002_json_parser.md) | Hand-written zero-dependency recursive descent parser | ✅ |
