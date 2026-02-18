# Architecture Rules | exile-ggpk
<!-- Type: Rule | Scope: exile-ggpk library | Lines: 75 max -->

## Library Contract
- This is a library crate — expose a clean, stable public API; hide implementation details
- Public API changes are breaking changes — treat with semver discipline
- No `main()`, no CLI, no application logic — pure library

## Safety Boundaries
- All `unsafe` FFI calls to ooz (C++ Oodle) are encapsulated in `ooz/` module
- Consumer code (exile-vision and others) must never touch raw FFI — use `ooz::decompress()`
- Every `unsafe` block requires a `// SAFETY:` comment explaining invariants maintained

## Error Handling
- No `unwrap()` or `expect()` in public API paths — propagate errors via `Result`
- Define domain error types with `thiserror` — no stringly-typed errors
- Malformed GGPK/bundle data must return `Err`, never panic

## Memory Strategy
- Large file reads use memory-mapped I/O (`memmap2`) — avoid large heap allocations
- Zero-copy parsing where possible — borrow slices from mapped memory
- Drop mapped memory explicitly when done; do not hold open across long operations

## Versioning Strategy
- Support both classic GGPK (pre-3.11.2) and bundle format (3.11.2+)
- Support both hash algorithms: FNV1a (legacy) and MurmurHash64A (3.21.2+)
- Format changes in PoE patches require a new ADR before implementation

## License Compliance
- GPL-3.0 — all new dependencies must be GPL-compatible
- Document license of each dependency in Cargo.toml comments or ADR
