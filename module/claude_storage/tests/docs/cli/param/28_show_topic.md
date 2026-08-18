# Parameter :: `show_topic::`

Edge case tests for the `show_topic::` parameter. Tests validate topic display from the first user message, newline flattening plus 90-character truncation, the off-default regression, non-boolean rejection, and combination with `since_days::`.

**Source:** [param/28_show_topic.md](../../../../docs/cli/param/28_show_topic.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `show_topic::1` appends first user message text | Display Format |
| EC-2 | Topic flattens newlines and truncates at 90 chars | Display Format |
| EC-3 | Omitted or `0` shows no topic text | Default |
| EC-4 | Non-boolean value rejected | Type Validation |
| EC-5 | Combined `since_days::` window plus topic display | Filter Interaction |

## Test Coverage Summary

- Display Format: 2 tests (EC-1, EC-2)
- Default: 1 test (EC-3)
- Type Validation: 1 test (EC-4)
- Filter Interaction: 1 test (EC-5)

**Total:** 5 edge cases

**Behavioral Divergence Pair:** EC-1 (`show_topic::1`, topic on session line) ↔ EC-3 (omitted/`0`, no topic text)

## Test Cases

---

### EC-1: `show_topic::1` appends first user message text

- **Commands:** `.projects`
- **Given:** A session whose first entry is a user message with known text
- **When:** `clg .projects scope::global show_topic::1`
- **Then:** The session's line (short ID, mtime, entry count) additionally carries the first user message text
- **Exit:** 0
- **Source:** [param/28_show_topic.md](../../../../docs/cli/param/28_show_topic.md)

---

### EC-2: Topic flattens newlines and truncates at 90 chars

- **Commands:** `.projects`
- **Given:** A session whose first user message is multi-line and longer than 90 characters
- **When:** `clg .projects scope::global show_topic::1`
- **Then:** The topic renders single-line (newlines become spaces) and exactly the first 90 characters appear; the 91st character is cut
- **Exit:** 0
- **Source:** [param/28_show_topic.md](../../../../docs/cli/param/28_show_topic.md)

---

### EC-3: Omitted or `0` shows no topic text

- **Commands:** `.projects`
- **Given:** Same fixture as EC-1
- **When:** `clg .projects scope::global` and `clg .projects scope::global show_topic::0`
- **Then:** The session is listed but the message text never appears — bare output is unchanged
- **Exit:** 0
- **Source:** [param/28_show_topic.md](../../../../docs/cli/param/28_show_topic.md)

---

### EC-4: Non-boolean value rejected

- **Commands:** `.projects`
- **Given:** clean environment
- **When:** `clg .projects show_topic::abc`
- **Then:** Coercion error on the `show_topic` argument (cannot coerce to Boolean)
- **Exit:** non-zero
- **Source:** [param/28_show_topic.md](../../../../docs/cli/param/28_show_topic.md)

---

### EC-5: Combined `since_days::` window plus topic display

- **Commands:** `.projects`
- **Given:** A 5-day-old session with a custom topic and a 25-day-old fixture session (whose topic would be `entry 0`)
- **When:** `clg .projects scope::global since_days::20 show_topic::1`
- **Then:** The windowed session appears with its topic; the excluded session contributes neither a line nor a topic
- **Exit:** 0
- **Source:** [param/28_show_topic.md](../../../../docs/cli/param/28_show_topic.md)
