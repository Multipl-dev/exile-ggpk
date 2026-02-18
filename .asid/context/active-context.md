# Active Context | Current Session State
<!-- Type: Session | Scope: Current work | Lines: 75 max -->
<!-- UPDATE: Every session start/end -->

---

## Current Phase

**FOUNDATION** (Active) | Library Created, API Surface Pending Audit

---

## Session 2 Complete (2026-02-03)

**Work:** Fork creation and library crate refactor

**Completed:**
- ✅ Forked ggpk-explorer (PoE community tool)
- ✅ Stripped ~4400 lines of UI/binary scaffolding
- ✅ Converted to library crate (lib.rs entry point)
- ✅ Modules intact: ggpk/, bundles/, dat/, ooz/ FFI wrapper

---

## Library State

| Module | Lines | Purpose |
|--------|-------|---------|
| ggpk/ | ~450 | Classic GGPK format (pre-3.11.2) |
| bundles/ | ~460 | Bundle format + Oodle decompression |
| dat/ | ~600 | .dat/.dat64 game data parsing |
| ooz/ | ~85 | FFI wrapper for C++ Oodle library |

---

## Next Priority

1. **Audit public API surface** — what does exile-vision actually need?
2. **Integration tests** — test against real PoE data files
3. **Crate publication** — crates.io when API is stable (GPL-3.0)

---

## Open Questions

- **API design:** streaming vs in-memory access? (TBD with exile-vision integration)
- **Versioning:** how to handle PoE format changes between patches?

---

## Repository State

```
exile-ggpk/ main @ f0f369d ✅ pushed
```

---

## Handoff

Session 2: Library crate created from fork (shared session with exile-vision S2)
- Full notes: exile-vision/.asid/context/handoff-2026-02-03-s2.md

Session 3 (2026-02-18): Full ASID bootstrap — .asid/, WEIR/, CLAUDE.md scaffolded
- Full notes: exile-ggpk/.asid/context/handoff-2026-02-18-s1.md (first independent handoff)
