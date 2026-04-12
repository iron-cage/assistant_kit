# json_schema

Supplies a JSON Schema that Claude's response must conform to.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--json-schema <schema>` |
| Env Var | — |
| Config Key | — |

### Type

json string

### Default

—

### Description

Supplies a JSON Schema that Claude's response must conform to. Claude attempts to produce output matching the schema structure. Useful for structured data extraction pipelines where the response must be machine-parseable JSON. Best combined with `--output-format=json`. Example: `{"type":"object","properties":{"name":{"type":"string"}},"required":["name"]}`.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |