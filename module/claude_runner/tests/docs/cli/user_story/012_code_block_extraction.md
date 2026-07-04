# User Story: Code Block Extraction

- **Source:** [docs/cli/user_story/012_code_block_extraction.md](../../../../docs/cli/user_story/012_code_block_extraction.md)
- **Primary flags:** `--strip-fences`
- **Command:** `run`

## Test Case Index

| ID | Category | Summary |
|----|----------|---------|
| US-1 | Happy path | `--strip-fences` removes outermost fence pair from stdout |
| US-2 | Boundary | No fence pair present — stdout passes through unmodified |
| US-3 | Parameter interaction | `--strip-fences` with `--file` for code generation pipeline |
| US-4 | Boundary | `--strip-fences` has no effect in `--dry-run` mode |

---

### US-1: strip outermost fence pair

- **Given:** Claude output contains a fenced code block
- **When:** `clr --strip-fences "Generate a hello world function" --dry-run`
- **Then:** Assembled command includes `--strip-fences`; at runtime, the outermost `` ``` `` lines (with optional language tag) would be removed, emitting only the interior content
- **Exit:** 0

### US-2: no fence pair — passthrough

- **Given:** Claude output has no markdown fence delimiters
- **When:** `clr --strip-fences "What is 2+2?" --dry-run`
- **Then:** Assembled command includes `--strip-fences`; at runtime, stdout would pass through unmodified since no fence pair is found (no-op behavior)
- **Exit:** 0

### US-3: strip-fences with file input

- **Given:** A readable file exists at a known path
- **When:** `clr --file /path/to/schema.json --strip-fences "Generate code from this schema" --dry-run`
- **Then:** Assembled command includes both `--file` and `--strip-fences`; at runtime, file bytes feed stdin and outermost fences are stripped from output
- **Exit:** 0

### US-4: strip-fences ignored in dry-run

- **Given:** No subprocess execution expected
- **When:** `clr --strip-fences --dry-run "Generate code"`
- **Then:** Assembled command includes `--strip-fences` but stripping is a post-processing step that only applies to subprocess output; dry-run shows the command without performing stripping
- **Exit:** 0
