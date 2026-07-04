# User Story: Structured JSON Pipeline

- **Source:** [docs/cli/user_story/013_structured_json_pipeline.md](../../../../docs/cli/user_story/013_structured_json_pipeline.md)
- **Primary flags:** `--json-schema`, `--strip-fences`
- **Command:** `run`

## Test Case Index

| ID | Category | Summary |
|----|----------|---------|
| US-1 | Happy path | `--json-schema` constrains output to given schema |
| US-2 | Happy path | `--json-schema` combined with `--strip-fences` for bare JSON |
| US-3 | Parameter interaction | `--json-schema` with `--file` for data extraction |
| US-4 | Boundary | Schema passed inline vs via shell substitution |

---

### US-1: schema-constrained output

- **Given:** A valid JSON Schema string
- **When:** `clr --json-schema '{"type":"object","properties":{"name":{"type":"string"}}}' --dry-run "Extract the author name"`
- **Then:** Assembled command includes `--json-schema` with the provided schema; Claude output would conform to the schema structure
- **Exit:** 0

### US-2: bare JSON with strip-fences

- **Given:** A valid JSON Schema string
- **When:** `clr --json-schema '{"type":"object"}' --strip-fences --dry-run "Extract data"`
- **Then:** Assembled command includes both `--json-schema` and `--strip-fences`; at runtime, output would be bare JSON without markdown fence delimiters, ready for `jq` piping
- **Exit:** 0

### US-3: file-driven JSON extraction

- **Given:** A readable data file and a JSON Schema
- **When:** `clr --file /path/to/data.txt --json-schema '{"type":"array"}' --strip-fences --dry-run "Extract entities"`
- **Then:** Assembled command includes `--file`, `--json-schema`, and `--strip-fences`; file content feeds stdin for schema-based extraction
- **Exit:** 0

### US-4: schema via shell substitution

- **Given:** Schema file at `/path/to/schema.json`
- **When:** `clr --json-schema "$(cat /path/to/schema.json)" --dry-run "Extract"`
- **Then:** Assembled command includes `--json-schema` with expanded schema content; functionally equivalent to inline string
- **Exit:** 0
