# Testing Rules | exile-ggpk
<!-- Type: Rule | Scope: exile-ggpk library | Lines: 75 max -->

## Coverage Requirements
- [ ] All public API entry points have at least one integration test
- [ ] Each module (ggpk, bundles, dat, ooz) has unit tests
- [ ] Error paths tested: malformed data, truncated files, invalid magic bytes

## Test Data Strategy
- Unit tests use synthetic in-memory byte arrays for speed and reproducibility
- Integration tests reference real PoE data files (gitignored, documented in tests/README.md)
- Malformed input corpus maintained in `tests/fixtures/malformed/`
- Tests must pass without real PoE data (skip integration tests if files absent)

## Rust Test Conventions
- Unit tests in `#[cfg(test)]` modules within each source file
- Integration tests in `tests/` directory at crate root
- `cargo test` must pass with zero warnings before any commit
- `cargo clippy -- -D warnings` must pass clean

## FFI Testing
- ooz decompression tested with known input/output pairs
- Malformed compressed data must return `Err`, not crash via C++ exception
- SAFETY invariants of `unsafe` blocks must be documented and verified in review

## TDD Requirement
- New format support: write failing test against sample data first, then implement
- Bug fixes: add regression test reproducing the bug before fixing (Article II)

## CI Gate
- `cargo test`, `cargo clippy -- -D warnings`, `cargo audit` must all pass
- No commit to main without all three green
