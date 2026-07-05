# Parameter :: `--creds`

Edge case test planning for the `--creds` parameter. See [019_creds.md](../../../../docs/cli/param/019_creds.md) for specification.

**Source:** [param/019_creds.md](../../../../docs/cli/param/019_creds.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Valid file path → accepted, trace shows creds path | Behavioral Divergence |
| EC-2 | Different valid path → trace shows different creds path | Behavioral Divergence |
| EC-3 | Relative path → resolved against caller's cwd | Valid |
| EC-4 | File does not exist → exit 1 with file-not-found error | Invalid |
| EC-5 | `--creds` without value → exit 1, argument requires value | Missing Value |
| EC-6 | `--creds` omitted, `CLR_CREDS` unset → defaults to `$HOME/.claude/.credentials.json` | Default Fallback |
| EC-7 | `--creds` omitted, `CLR_CREDS` set → `CLR_CREDS` value used (wins over default) | Default Fallback |
| EC-8 | `--creds` omitted, `CLR_CREDS` unset, `HOME` unset → exit 1 with error | Default Fallback: No HOME |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-1, EC-2)
- Valid: 1 test (EC-3)
- Invalid: 1 test (EC-4)
- Missing Value: 1 test (EC-5)
- Default Fallback: 3 tests (EC-6, EC-7, EC-8)

**Total:** 8 test cases

---

### EC-1: Valid file path → accepted, trace shows creds path

- **Given:** credentials JSON at `/tmp/ec1_creds.json`
- **When:** `clr isolated --creds /tmp/ec1_creds.json --trace "test"`
- **Then:** file found; trace stderr contains `# creds: /tmp/ec1_creds.json`; subprocess attempt exits 1 (claude absent in test environment)
- **Exit:** 1
- **Source:** [019_creds.md](../../../../docs/cli/param/019_creds.md)
- **Commands:** isolated, refresh

---

### EC-2: Different valid path → trace shows different creds path

- **Given:** credentials JSON at absolute path `/tmp/ec2_creds.json`
- **When:** `clr isolated --creds /tmp/ec2_creds.json --trace "test"`
- **Then:** trace stderr contains `# creds: /tmp/ec2_creds.json` (different from EC-1); subprocess attempt exits 1 (claude absent in test environment)
- **Exit:** 1
- **Source:** [019_creds.md](../../../../docs/cli/param/019_creds.md)
- **Commands:** isolated, refresh

---

### EC-3: Relative path → resolved against caller's cwd

- **Given:** credentials file `ec3_creds.json` in caller's cwd
- **When:** `clr isolated --creds ec3_creds.json "test"` (run from that directory)
- **Then:** file found via cwd resolution, not via isolated temp HOME; subprocess attempt exits 1 (claude absent in test environment)
- **Exit:** 1
- **Source:** [019_creds.md (Note: path resolved against caller's cwd)](../../../../docs/cli/param/019_creds.md)
- **Commands:** isolated, refresh

---

### EC-4: File does not exist → exit 1 with file-not-found error

- **Given:** `/tmp/ec4_nonexistent.json` does not exist
- **When:** `clr isolated --creds /tmp/ec4_nonexistent.json "test"`
- **Then:** exit 1; stderr contains file-not-found error; no subprocess launched
- **Exit:** 1
- **Source:** [019_creds.md](../../../../docs/cli/param/019_creds.md)
- **Commands:** isolated, refresh

---

### EC-5: `--creds` without value → exit 1, argument requires value

- **Given:** clean environment
- **When:** `clr isolated --creds`
- **Then:** exit 1; stderr says `--creds` requires a value
- **Exit:** 1
- **Source:** [019_creds.md](../../../../docs/cli/param/019_creds.md)
- **Commands:** isolated, refresh

---

### EC-6: `--creds` omitted, `CLR_CREDS` unset → trace confirms default path

- **Given:** `$HOME/.claude/.credentials.json` exists (readable; content `{}`); `CLR_CREDS` unset
- **When:** `clr isolated --trace "test"` (no `--creds`)
- **Then:** trace stderr contains `# creds: <HOME>/.claude/.credentials.json`; subprocess attempt fails (claude absent in test environment)
- **Exit:** 1
- **Source:** [019_creds.md](../../../../docs/cli/param/019_creds.md)
- **Commands:** isolated, refresh

---

### EC-7: `CLR_CREDS` set, `--creds` omitted → `CLR_CREDS` wins over default

- **Given:** `CLR_CREDS=/tmp/ec7_creds.json` set; `/tmp/ec7_creds.json` exists
- **When:** `clr isolated --trace "test"` (no `--creds`)
- **Then:** trace stderr contains `# creds: /tmp/ec7_creds.json` (not the HOME default); subprocess attempt fails (claude absent in test environment)
- **Exit:** 1
- **Source:** [019_creds.md](../../../../docs/cli/param/019_creds.md)
- **Commands:** isolated, refresh

---

### EC-8: `--creds` omitted, `CLR_CREDS` unset, `HOME` unset → exit 1

- **Given:** `HOME` unset in subprocess environment; `CLR_CREDS` unset; `--creds` not provided
- **When:** `clr isolated "test"`
- **Then:** exit 1; stderr contains an error (cannot resolve default path without HOME); no subprocess launched
- **Exit:** 1
- **Source:** [019_creds.md](../../../../docs/cli/param/019_creds.md)
- **Commands:** isolated, refresh
