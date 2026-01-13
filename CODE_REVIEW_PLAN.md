# Code Review Implementation Plan

## Dead Code Removal

### Priority 1: Remove unused files and code
- [x] Delete `src/handlers/stateful.rs` - never instantiated
- [x] Delete `src/handlers/static_handler.rs` - never used
- [x] Delete `src/handlers/base.rs` - Handler trait only used by dead code
- [x] Remove `error_handler` from `src/middleware/error.rs` - never wired in
- [x] Remove unused error variants from `src/error.rs`
- [x] Clean up `src/handlers/mod.rs` exports
- [x] Remove duplicate `contains()` method from `src/handlers/custom.rs`

### Priority 2: Remove unused parameters
- [x] Remove `_mode` parameter from `build_router()` in `server/router.rs`
- [x] Address unused `_event` parameters in webhook handlers (kept with `_` prefix as intentional)

## Performance Fixes

### Priority 1: Critical bottlenecks
- [x] Cache regex in `openapi/parser.rs` using `std::sync::LazyLock`
- [x] Add token index by `access_token` in `state/auth.rs` for O(1) lookup
- [ ] Index webhooks by tenant in `state/webhooks.rs` (optional - low priority)

### Priority 2: Efficiency improvements
- [x] Replace excessive `.clone()` calls with references where possible (fixed via clippy)
- [ ] Store projects directly in hub_projects map in `state/projects.rs` (optional)

## Bug Fixes

### Priority 1: Security/correctness
- [x] Wire up actual token validation in `middleware/auth.rs`

### Priority 2: Robustness
- [x] Replace `.unwrap()` calls with proper error handling
- [ ] Return error instead of empty vec for non-existent OpenAPI dir (design decision)

## Code Quality

### Clippy fixes applied:
- [x] Use `#[derive(Default)]` with `#[default]` attribute for MockMode
- [x] Use `or_default()` instead of `or_insert_with(Default::new)`
- [x] Use `next_back()` instead of `last()` for DoubleEndedIterator
- [x] Use `std::io::Error::other()` for creating IO errors
- [x] Collapse nested if statements where appropriate
- [x] Box large enum variants to reduce size (Parameter::Definition)

## Summary

All critical issues have been resolved:
- 3 dead files removed
- 1 unused middleware function removed
- 5 unused error variants removed
- Regex now cached at compile time
- Token validation now O(1) instead of O(n)
- Auth middleware now actually validates tokens
- All clippy warnings resolved
- Code formatted with rustfmt
