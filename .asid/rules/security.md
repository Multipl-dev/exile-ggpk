# Security Rules | exile-ggpk
<!-- Type: Rule | Scope: exile-ggpk library | Lines: 75 max -->

## Rust Safety
- [ ] All `unsafe` blocks must have `// SAFETY:` documentation
- [ ] FFI boundary (ooz C++ calls) has a safe Rust wrapper — no raw FFI exposed publicly
- [ ] No `unwrap()` or `expect()` in library code — callers receive `Result`
- [ ] Run `cargo audit` before any release to check for known CVEs in dependencies

## File System
- [ ] Validate file paths before memory-mapping — reject obviously invalid input
- [ ] Handle truncated or malformed GGPK/bundle data gracefully (return `Err`)
- [ ] No file writes — this is a read-only library
- [ ] Memory-mapped file handles closed promptly after use

## Dependency Policy
- [ ] New dependencies require license audit (GPL-3.0 compatibility)
- [ ] Prefer well-maintained crates with no known CVEs
- [ ] Native dependencies (C/C++) require explicit security review
- [ ] Pin dependency versions in Cargo.lock for reproducible builds

## Review Triggers
Changes to ooz FFI boundary, file parsing, or any `unsafe` code require security review.
