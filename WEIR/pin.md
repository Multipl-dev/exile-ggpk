# WEIR | Error Catchall and Incident Documentation

This folder contains mission-critical error logs for troubleshooting.

**This WEIR sits downstream of the work, gathering whatever slips past daily attention (much as a weir across a river catches fish and debris): errors, oversights, patterns, and lessons that would otherwise drift away can be found here.**

---

## Workflow Rules

1. **Check local first.** Read local file before git pull - user may have uncommitted content.
2. **git status before git pull.** Verify working tree state before pulling.

---

## Debugging Rules

1. **cargo check before cargo build.** Type errors surface faster with check.
2. **RUST_BACKTRACE=1** for runtime panics — always include full backtrace in ERR log.
3. **Multiple issues coexist.** First fix may not resolve symptom if other issues remain.
4. **Read the error chain.** Rust errors show root cause at the bottom of the chain.
5. **ooz submodule.** If build fails with missing C headers: `git submodule update --init`.

---

## Platform-Specific (Rust / FFI / PoE)

1. **ooz submodule must be initialized.** `git submodule update --init` after fresh clone.
2. **C++17 required.** Ensure compiler toolchain supports C++17 before building ooz.
3. **bindgen regenerates on clean.** FFI bindings are generated at build time — not checked in.
4. **PoE patches may change format.** Verify library compatibility after major PoE updates.

---

## File Conventions

| File | Purpose | Lifecycle |
|------|---------|-----------|
| `pin.md` | Standing rules | Permanent |
| `ERR` | Current error log | Replaced per incident |
| `POSTMORTEM_NNN.md` | Incident analyses | Preserved |

---

## Error Logging Protocol

When error occurs:
1. Log to `WEIR/ERR` with timestamp and full error
2. Invoke `debug-error-tracer` agent
3. After resolution, create `WEIR/POSTMORTEM_NNN.md`
4. Clear `ERR` for next incident
