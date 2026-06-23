# Parameter 062: `owner::` — Edge Cases

### Test Case Index

| ID | Test | Scenario | Expected | Status |
|----|------|----------|----------|--------|
| EC-01 | `ec1_owner_sets_custom_identity` | `owner::alice@laptop name::X` | writes `"owner": "alice@laptop"` to `{name}.json` | ✅ |
| EC-02 | `ec2_owner_empty_rejected` | `owner::` with empty value | exit 1 with "use unclaim::1 to clear" | ✅ |
| EC-03 | `ec3_owner_and_unclaim_mutual_exclusion` | `owner::user1@w003 unclaim::1 name::X` | exit 1 mutual exclusion | ✅ |
| EC-04 | `ec4_owner_missing_name_exits_1` | `owner::user1@w003` (no name::) | exit 1 with usage error | ✅ |
| EC-05 | `ec5_owner_g8_foreign_owner_blocked` | account owned by `other@host`, caller is not `other@host` | exit 1 ownership violation | ✅ |
| EC-06 | `ec6_owner_force_bypasses_g8` | same as EC-05 + `force::1` | write succeeds, exit 0 | ✅ |
| EC-07 | `ec7_owner_dry_no_file_writes` | `owner::user1@w003 name::X dry::1` | `[dry-run]` message, no `{name}.json` change | ✅ |
| EC-08 | `ec8_owner_overwrite_existing` | account already owned by caller → `owner::new@identity` | overwrites to new identity | ✅ |
| EC-09 | `ec9_owner_idempotent_same_value` | `owner::user1@w003` when already `owner: "user1@w003"` | no-op write, exit 0 | ✅ |

**Total:** 9 edge case tests
