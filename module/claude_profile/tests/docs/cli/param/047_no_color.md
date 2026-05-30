# Test: `no_color::` Parameter

Edge case coverage for the `no_color::` parameter on `.usage`. See [param/047_no_color.md](../../../../docs/cli/param/047_no_color.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `no_color::1` output contains no emoji characters | Behavioral Divergence |
| EC-2 | `no_color::1` status column shows text labels | Status Labels |
| EC-3 | `no_color::0` (default) output includes emoji | Behavioral Divergence |
| EC-4 | `no_color::bad` exits 1 naming valid values | Invalid Value |
| EC-5 | `no_color::1` footer uses ASCII `->` instead of unicode arrow | ASCII Footer |
| EC-6 | `no_color::true` accepted (alias for 1) | Alias Acceptance |

---

### EC-1: `no_color::1` output contains no emoji

- **Given:** One 🟢 account.
- **When:** `clp .usage no_color::1`
- **Then:** Exits 0. Stdout contains none of: `🟢`, `🟡`, `🔴`, `→`, `✓`, `*` (emoji/special markers). No ANSI escape sequences.
- **Exit:** 0
- **Source fn:** `it198_no_color_1_no_emoji_in_output` (in `tests/cli/usage_test.rs`)
- **Source:** [param/047_no_color.md](../../../../docs/cli/param/047_no_color.md)

---

### EC-2: `no_color::1` status column shows plain text label

- **Given:** One 🟢 account.
- **When:** `clp .usage no_color::1`
- **Then:** Exits 0. Status column (●) shows text label `ok` instead of `🟢`.
- **Exit:** 0
- **Source fn:** `it199_no_color_1_status_shows_err_text_label` (in `tests/cli/usage_test.rs`)
- **Source:** [param/047_no_color.md](../../../../docs/cli/param/047_no_color.md)

---

### EC-3: `no_color::0` output includes emoji (default)

- **Given:** One 🟢 account.
- **When:** `clp .usage no_color::0`
- **Then:** Exits 0. Stdout contains `🟢` status emoji (default behavior with color).
- **Exit:** 0
- **Live:** yes
- **Source fn:** ⏳ (in `tests/cli/usage_test.rs`)
- **Source:** [param/047_no_color.md](../../../../docs/cli/param/047_no_color.md)

---

### EC-4: `no_color::bad` exits 1 naming valid values

- **Given:** Any environment.
- **When:** `clp .usage no_color::bad`
- **Then:** Exits 1. Stderr names valid values: `0`, `1`, `false`, `true`.
- **Exit:** 1
- **Source fn:** `it200_no_color_bad_exits_1` (in `tests/cli/usage_test.rs`)
- **Source:** [param/047_no_color.md](../../../../docs/cli/param/047_no_color.md)

---

### EC-5: `no_color::1` footer uses ASCII `->` instead of unicode arrow

- **Given:** Two accounts with valid quota (footer shown).
- **When:** `clp .usage no_color::1`
- **Then:** Exits 0. Footer lines use `->` (ASCII) for strategy labels. Unicode arrow `→` not present in footer.
- **Exit:** 0
- **Source fn:** ⏳ (in `tests/cli/usage_test.rs`)
- **Source:** [param/047_no_color.md](../../../../docs/cli/param/047_no_color.md)

---

### EC-6: `no_color::true` accepted as alias for 1

- **Given:** One 🟢 account.
- **When:** `clp .usage no_color::true`
- **Then:** Exits 0. No emoji in output — same result as `no_color::1`.
- **Exit:** 0
- **Source fn:** `it201_no_color_true_accepted` (in `tests/cli/usage_test.rs`)
- **Source:** [param/047_no_color.md](../../../../docs/cli/param/047_no_color.md)
