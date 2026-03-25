## Summary

Brief description of what this PR does and why.

## Changes

-
-
-

## Test Plan

- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy -- -W clippy::all -D clippy::unwrap_used -D clippy::expect_used` passes
- [ ] `cargo test` passes
- [ ] Tested on target hardware (specify board):
- [ ] Verified graceful degradation on non-Pi Linux (if applicable)

## Checklist

- [ ] No `unwrap()` or `expect()` in production code paths
- [ ] No hardcoded hwmon numbers
- [ ] All sysfs reads handle missing files gracefully
- [ ] New collectors implement the `Collector` trait
- [ ] UI changes work at 80x24 minimum terminal size
- [ ] Added/updated unit tests for new functionality
