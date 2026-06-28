# 006_doc_entity_index_consistency

Test spec for `docs/invariant/006_doc_entity_index_consistency.md`.

## Overview

| Case ID | Category | Status |
|---------|----------|--------|
| DEI-1 | Instance count accuracy | ⏳ |
| DEI-2 | Listed file existence | ⏳ |

## Cases

### DEI-1: entity.md instance counts match NNN file counts on disk

- **Given:** All `entity.md` files in `docs/` and `module/*/docs/` (12 files total: workspace root + 11 crate-level)
- **When:** For each entity row in the Master Doc Entities Table, the `Instances` value is read and the corresponding entity directory is scanned for `NNN_*.md` files (files matching `^[0-9]{3}_.*\.md`, excluding `readme.md` and `procedure.md`)
- **Then:** The count from the directory scan equals the `Instances` value in `entity.md` for every entity row across all 12 files; any discrepancy is reported as `{crate}/{entity}: expected {count} got {actual}`

### DEI-2: All files listed in Master Doc Instances Table exist on disk

- **Given:** All `entity.md` files in `docs/` and `module/*/docs/`
- **When:** Every file path in the Master Doc Instances Table `File` column is resolved relative to the `entity.md` parent directory and checked for existence
- **Then:** All resolved paths exist as regular files; any missing path is reported as `{crate}/{entity}/{file}: not found`
