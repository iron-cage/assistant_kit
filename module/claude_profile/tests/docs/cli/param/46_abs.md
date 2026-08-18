# REMOVED — `abs::` Parameter

> **REMOVED:** The `abs::` parameter has been removed. It was registered as a no-op
> (the absolute-token-count display was never implemented — output was identical with
> or without it), so removal needs no migration. Passing `abs::` to `.usage`/`.accounts`
> now fails as an unknown parameter.

No rejection test: unlike `next::` (which kept a registered migration-error stub asserted
by `it253_next_param_removed_exit_1`), `abs::` is fully unregistered — rejection is the
framework's generic unknown-parameter behavior, not command-specific logic to guard.
The former EC tests (`it194`–`it197`, `it223`–`it224`) were deleted along with the parameter.
