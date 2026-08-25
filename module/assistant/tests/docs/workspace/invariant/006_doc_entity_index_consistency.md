# 006_doc_entity_index_consistency

Test spec for `docs/invariant/006_doc_entity_index_consistency.md`.

## Overview

| Case ID | Category | Status |
|---------|----------|--------|
| DEI-1 | Instance count accuracy | ✅ |
| DEI-2 | Listed file existence | ✅ |

## Cases

### DEI-1: registry instance counts match instance file counts on disk

- **Given:** Every doc entity registry in `docs/` and `module/*/docs/`, in both shapes — `entity.md` (12: workspace root + 11 crate-level) and `entity/readme.md` (3: `claude_journal`, `claude_journal_viewer`, `claude_profile`)
- **When:** For each entity row in the Master Doc Entities Table, the `Instances` value is read and the entity directory — resolved as the parent of the row's own Master File link target — is scanned for instance files (every `*.md` excluding `readme.md` and `procedure.md`; prefix shape varies by entity family per its own design ruleset and is not policed here)
- **Then:** The count from the directory scan equals the `Instances` value for every entity row across all 15 registries; any discrepancy is reported as `{registry_path}/{entity}: expected {count} got {actual}`
- **Note:** the three `entity/readme.md` registries were outside the discovery pattern until it was widened, and produced 8 divergences the moment they came into scope — a green suite that had never read them. The widening is what makes the count of registries (15, not 12) load-bearing rather than incidental

### DEI-2: All files listed in Master Doc Instances Table exist on disk

- **Given:** Every doc entity registry in `docs/` and `module/*/docs/`, in both shapes
- **When:** Every file path in the Master Doc Instances Table `File` column is resolved relative to the registry's parent directory and checked for existence
- **Then:** All resolved paths exist as regular files; any missing path is reported as `{registry_path} → {file}: not found`
- **Note:** the `entity/readme.md` shape resolves one level deeper than `entity.md`, so its rows carry a `../` prefix the flat shape does not. DEI-2 passed on all three of them on first contact, which is what confirms the deeper resolution works rather than silently missing every path
