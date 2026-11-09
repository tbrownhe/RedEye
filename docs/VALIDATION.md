# RedEye validation evidence

## Milestone 1 — 2026-11-08

Status: milestone 1 complete on the tested Windows 11 environment. Independent
code review and automated checks passed. The user externally observed baseline,
active/off/on transitions, a maintained request, and removal after normal exit.
Later milestones have not been validated.

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
| Elevated active/off/on request transitions | User supplied three outputs in requested order: active and on show RedEye SYSTEM request; off shows no requests; DISPLAY remains `None` | Pass: user-observed transitions |
| Request maintained after 30 seconds | User's next requested diagnostic shows the same RedEye SYSTEM request; DISPLAY and other categories `None` | Pass: user-observed |
| Request removed after `quit` | User's final requested diagnostic has every category `None` | Pass: user-observed |

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

The user supplied elevated `powercfg /requests` outputs for active, off, and on,
in that requested order. Active and on each list this SYSTEM request:

```text
[PROCESS] \Device\HarddiskVolume3\Users\tbrow\dev\RedEye\target\debug\redeye.exe
```

DISPLAY, AWAYMODE, EXECUTION, PERFBOOST, and ACTIVELOCKSCREEN are `None` in
those active/on outputs. The off output has every category `None`. The user
then supplied the requested 30-second maintained and post-quit outputs in that
order: the first retains the same SYSTEM process request with DISPLAY and other
categories `None`; the second has every category `None`. These external
observations establish the milestone 1 native gate on Windows 11. The executable
was unchanged, so the passing checks and recorded hash were reused.

## Native milestone 1 procedure

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

This procedure is complete through user-observed diagnostics for the artifact
identified above. The agent's own diagnostic could not run without administrator
privileges; user evidence is distinguished from agent-run checks in the table.

Ctrl+C, console closure, forced termination, locking, explicit sleep, Windows
10 coverage, and long-duration idle behavior were not tested. Tray interactions,
display preference, persistence, startup registration, single-instance behavior,
icons/DPI, Explorer restart, and portable release checks belong to later stages.

See [the workflow](WORKFLOW.md) for gates and
[the README](../README.md) for current console usage.

## Original icon assets — 2026-11-08

Independent asset review found no defects. The validator regenerated the frozen
assets using the documented command below; every file was byte-identical before
and after. The argument changes execution policy only for that child process.
No application code or other generated files were modified.

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools/generate-icons.ps1
```

| Artifact | Bytes | SHA256 before and after regeneration |
|---|---:|---|
| `assets/redeye-active.ico` | 66,197 | `A0E1B234D2A6EA01B76156EA8CB371595F413F0A69374FCBE4FE3FE0F25D3077` |
| `assets/redeye-inactive.ico` | 45,303 | `1973E8BA8FD974AECDF4CD92F744B36CA4FD8F41EAE46190379706D7501942D1` |
| `assets/tray-preview.png` | 24,654 | `10A6231599233948AF1C1AA37E5754FE55A09BACF4398B29FEC65A3A818AF38D` |

Both ICO directories contain exactly eight frames: 16, 20, 24, 32, 48, 64,
128, and 256 pixels square. Binary checks passed for ICO header/type, one plane,
32-bit directory entries, contiguous nonoverlapping payload bounds, and no
trailing bytes. Every payload has the PNG signature and 8-bit RGBA color type;
System.Drawing decoded all sixteen frames at the declared dimensions. Each
frame's top-left corner is transparent.

The 352x256 preview decoded successfully and was visually inspected at its
original resolution. Active has a clear bright red lens/core; inactive has a
smaller matte burgundy lens and dark core. Both remain distinguishable at the
shown 16/20/24/32 pixel sizes on light and dark backgrounds. All local links in
[the icon documentation](ICONS.md) exist. This validates asset files and a local
preview, not native notification-area readability, DPI scaling, or monitor
changes; those checks remain pending native tray integration.

## Milestone 2 — 2026-11-08

Status: milestone 2 complete on the tested Windows 11 environment. Independent
integrated code review found no defects; automated checks passed. The user
confirmed tray/menu interactions and supplied external power-request evidence.
No second interactive process was launched by the validator while the user tested.

Environment is the Windows 11 x64/Rust 1.99.1 MSVC environment identified above.
Artifact: frozen working-tree milestone 2, `target/debug/redeye.exe`, SHA256
`702EB2C89AC898E2EF9C5184D3097385B33B603CBE3081A7134DF9BAEE72D3B7`.
The hash matched before and after validation. The lockfile contains `redeye`,
`windows-sys 0.61.2`, and its sole dependency `windows-link 0.2.1`.

| Check | Actual result | Status |
|---|---|---|
| `cargo fmt --check` | Exit 0 | Pass |
| `cargo check --locked` | Exit 0 | Pass |
| `cargo clippy --locked --all-targets -- -D warnings` | Exit 0, no lint findings | Pass |
| `cargo test --locked` | Three tests passed: power flags, failed transitions, embedded-icon frames/corrupt bounds | Pass |
| `cargo build --locked` | Exit 0; expected executable hash unchanged | Pass |
| Existing icon artifacts | All three hashes still match the asset section; no asset changes | Pass: reused asset evidence |
| README and local documentation links | Current usage describes native tray/Exit and diagnostic-only left click; all local links exist; `git diff --check` has no whitespace errors | Pass |
| Native initialization/message shutdown smoke | Implementer reported an owned outside-sandbox process, hidden HWND matched to its PID, `WM_CLOSE`, exit 0, Awake and explicit-clear output, empty stderr | Pass: supplied smoke evidence only |
| Icon/tooltip/native menu/dismiss/Exit | User confirmed the requested procedure, including extra clicks; icon/tooltip/menu and normal Exit behaved as expected | Pass: user-observed |
| Repeated left/right event distinction | User console output alternated left/right events three times; each left/right action left awake state unchanged as expected for milestone 2 | Pass: user-observed |
| Native requests before and after Exit | Before Exit: RedEye SYSTEM process request only; after Exit: every category `None`; console reported explicit clear | Pass: user-observed |

The implementer's default-sandbox attempt could not establish an accessible
hidden window and was not counted as a successful native test. Only its own
verified executable child was stopped. The later outside-sandbox smoke above
finished before the user-test handoff. It proves initialization and message
shutdown, not mouse/menu behavior or external Windows request removal. Separate
user evidence completes those native checks: the user confirmed the requested
procedure while noting extra clicks. Console output showed the initial Awake
instruction, three alternating left/right event pairs with state unchanged,
and final `RedEye: execution-state request cleared.` Before Exit, elevated
`powercfg /requests` listed SYSTEM:

```text
[PROCESS] \Device\HarddiskVolume3\Users\tbrow\dev\RedEye\target\debug\redeye.exe
```

DISPLAY and all other categories were `None`; after Exit, all categories were
`None`. The same frozen executable hash was retained, so automated checks were
not repeated after this user evidence.

### Completed native procedure

1. In Administrator PowerShell, capture `powercfg /requests` before launch.
2. Launch the debug executable in a normal PowerShell. Find the original red
   eye icon in the notification area or overflow; hover for `RedEye — Awake`.
3. Run the diagnostic. Expect RedEye's SYSTEM request and no RedEye DISPLAY
   request.
4. Left-click once. Expect the console diagnostic and unchanged awake state
   in this milestone. Right-click and dismiss the native menu without choosing
   Exit. Expect no awake transition or extra left-click diagnostic from the
   right-click action. Repeat the power diagnostic; expect the same request.
5. Right-click and choose Exit. Expect icon removal, explicit-clear output,
   and process exit. Repeat the diagnostic; expect RedEye's request absent.

The user completed this procedure for the artifact above, with extra repeated
clicks as described in the evidence. Retain these observations and labeled diagnostics.
Actual-size icon/DPI, Explorer restart, Windows 10, forced termination, locking,
explicit sleep, and release-only checks remain unperformed. Interactive state,
display preference, settings, startup, and single-instance checks are later
milestones, not milestone 2 acceptance claims.
