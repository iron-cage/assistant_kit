# Feature 063 — Explicit Ownership Claim

### Test Case Index

| ID | Test | Verifies | Status |
|----|------|----------|--------|
| FT-01 | `ft_owner_sets_owner_field` | AC-01: `owner::user1@w003 name::X` writes owner field | ✅ |
| FT-02 | `ft_owner_requires_name` | AC-02: exits 1 when `name::` absent | ✅ |
| FT-03 | `ft_owner_g8_blocks_non_owner` | AC-03: G8 gate — owned by another → exit 1 | ✅ |
| FT-04 | `ft_owner_unowned_passes_g8` | AC-04: unowned account → write succeeds | ✅ |
| FT-05 | `ft_owner_mutual_exclusion_unclaim` | AC-05: `owner:: + unclaim::1` → exit 1 | ✅ |
| FT-06 | `ft_owner_dry_run_preview` | AC-06: dry::1 → preview, no file writes | ✅ |
| FT-07 | `ft_owner_force_bypasses_g8` | AC-07: force::1 bypasses G8 for other-owned account | ✅ |
| FT-08 | `ft_owner_trace_emits_diagnostic` | AC-08: trace::1 → stderr diagnostic | ✅ |
| FT-09 | `ft_owner_prefix_resolution` | AC-09: short name resolves to full email | ✅ |
| FT-10 | `ft_owner_empty_value_rejected` | AC-10: empty owner:: → exit 1 | ✅ |
| FT-11 | `ft_owner_gates_respect_new_value` | AC-11: subsequent ops respect new owner | ✅ |
| FT-12 | `ft_owner_works_on_usage` | AC-12: `.usage owner::` same behavior as `.accounts owner::` | ✅ |

**Total:** 12 test cases (12 FT)
