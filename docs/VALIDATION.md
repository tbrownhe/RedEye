# RedEye validation evidence

## Milestone 1 — 2026-11-08

Status: implementation and independent code review complete; automated checks
passed. External Windows request appearance/removal remains pending. Later
milestones have not been validated.

Environment: Windows 11 Home 10.0.26300, build 26300, x64 (lead's native CIM
query); Rust 1.99.1 (`58cf8f9c3`), Cargo 1.99.1, native
`x86_64-pc-windows-msvc`. Rust/Cargo resolved without PATH changes or tool
installation. Windows 10 has not been tested.

Artifact: working-tree milestone 1, `target/debug/redeye.exe`, SHA256
`D27C5BBD653022C1B9069C4AC3CE4138FAFC6C4FAC76CC8E0884B86212300E06`.
The validator checked this hash before and after console smoke tests. The
lockfile contains only `redeye 0.1.0`; there are no third-party dependencies.

| Check | Actual result | Status |
|---|---|---|
| `cargo fmt --check` | Exit 0 | Pass |
| `cargo check --locked` | Exit 0 | Pass |
| `cargo clippy --locked --all-targets -- -D warnings` | Exit 0, no lint findings | Pass |
| `cargo test --locked` | Two tests passed: four flag combinations and failed activation/deactivation state retention | Pass |
| `cargo build --locked` | Exit 0, native debug executable | Pass |
| Console `status`, `off`, `status`, `on`, `status`, `quit` | Correct success/status messages, explicit final clear, exit 0 | Pass: API-return/UI evidence |
| Blank input followed by EOF | Explicit final clear, exit 0 | Pass: API-return evidence |
| Wait with stdin open | Owned child stayed alive; 0 ms CPU increase across a 3-second sample after a 1-second settling period; working set 4,005,888 bytes | Pass: short idle observation only |
| README usage and local links; `git diff --check` | Usage agrees with console; linked local files exist; no whitespace errors | Pass |
| Elevated `powercfg /requests` baseline | User reported all categories `None` before launch | Pass: user-observed baseline |
| Active, cleared, reactivated, maintained, and exited requests | Awaiting user diagnostic output; agent diagnostic requires administrator privileges | Not yet verified |

Rustup emitted its existing home-path canonicalization warning; all listed
commands succeeded. Console smoke tests ran a standard-user executable with
redirected input/output. Its owning main thread remained alive waiting for input
and exited through `quit`; only the validator's own child was controlled.
The short CPU sample is not a long-duration resource benchmark.

Successful API return values, unit tests, and console output do not prove that
Windows is honoring or releasing requests. The focused failure test injects
zero returns; it does not induce a real Windows API failure. Independent review
found no correctness defects in thread ownership, flags, commit-after-success,
FFI, or explicit cleanup paths.

## Remaining native milestone 1 check

Keep RedEye in a normal PowerShell; use a separate **Administrator PowerShell**
for the diagnostic. Elevation applies to the diagnostic, not the application.

1. Before launch, run `powercfg /requests` and save the baseline (user supplied:
   all categories `None`).
2. In normal PowerShell, run:

   ```powershell
   & 'C:\Users\tbrow\dev\RedEye\target\debug\redeye.exe'
   ```

3. In Administrator PowerShell, run `powercfg /requests`. Expect a SYSTEM
   request attributable to this RedEye executable and no RedEye DISPLAY request.
4. Enter `off` in RedEye. Repeat the diagnostic. Expect RedEye's request absent
   while its console remains open.
5. Enter `on`. Repeat the diagnostic immediately and after 30 seconds without
   another command to RedEye. Expect its SYSTEM request present on both checks,
   with no RedEye DISPLAY request.
6. Enter `quit`. Confirm the explicit-clear message and process exit; repeat
   the diagnostic. Expect RedEye's request absent.
7. Retain labeled outputs and the executable hash with the result. Distinguish
   unrelated applications' requests from RedEye's contribution.

Ctrl+C, console closure, forced termination, locking, explicit sleep, Windows
10 coverage, and long-duration idle behavior were not tested. Tray interactions,
display preference, persistence, startup registration, single-instance behavior,
icons/DPI, Explorer restart, and portable release checks belong to later stages.

See [the workflow](WORKFLOW.md) for gates and
[the README](../README.md) for current console usage.
