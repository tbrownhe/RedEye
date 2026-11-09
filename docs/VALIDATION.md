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

## Milestone 3 — 2026-11-08

Status: milestone 3 complete on the tested Windows 11 environment. Independent
final code review found no defects; automated checks passed. The user confirmed
the requested UI/click procedure and supplied the full native request matrix.
The validator did not launch a process or send UI input during the user test.

Environment: the same Windows 11 Home build 26300 x64 / Rust 1.99.1 MSVC
environment. Artifact: frozen working-tree milestone 3,
`target/debug/redeye.exe`, SHA256
`806A86B7A6312D33DDF46661693D0AC84DAEA48451D358E54E69F9B47B534327`.
The hash matched before and after checks. The unchanged lockfile still contains
only the app, `windows-sys 0.61.2`, and `windows-link 0.2.1`; unchanged original
icon evidence is reused.

| Check | Actual result | Status |
|---|---|---|
| `cargo fmt --check` | Exit 0 | Pass |
| `cargo check --locked` | Exit 0 | Pass |
| `cargo clippy --locked --all-targets -- -D warnings` | Exit 0, no lint findings | Pass |
| `cargo test --locked` | Four tests passed: flag matrix, failed awake transitions, failed display transitions, embedded-icon bounds | Pass |
| `cargo build --locked` | Exit 0; expected artifact hash unchanged | Pass |
| README usage, local documentation links, whitespace | README matches left-click/menu toggles, display preference, and nonpersistent milestone 3 behavior; all local links exist; `git diff --check` clean | Pass |
| Native four-state matrix | Eight ordered user diagnostics match the transitions recorded below | Pass: user-observed |
| Icons/tooltips/checkmarks, left/menu toggles, right-click dismissal | User confirmed the requested procedure; requested UI and single-action behavior remained consistent | Pass: user-observed |
| Exit cleanup | Final diagnostic has no RedEye request; console reported explicit clear | Pass: user-observed |

Final independent review traced typed tray events back to the main-thread power
owner. OS success precedes committing the display preference and rendering icon,
tooltip, and checkmarks. A recoverable power failure retains the last successful
state and visuals; a render failure exits through explicit tray and power
cleanup. Tests inject zero API returns for failure handling; no real native API
failure was induced or claimed. There is no persistence or single-instance
implementation in this milestone.

The user supplied eight `powercfg /requests` outputs in the requested order.
All listed requests identify the same RedEye debug executable path recorded for
milestone 2; unrelated categories remained `None`:

| Step | User-observed RedEye requests |
|---|---|
| Launch, active/display off | SYSTEM only |
| Display on while active | SYSTEM and DISPLAY |
| Disable while display preference on | None |
| Display off while inactive | None |
| Display on while inactive | None |
| Enable with display preference on | SYSTEM and DISPLAY |
| Display off while active | SYSTEM only |
| Exit | None |

Console output agreed: initial Awake/display may turn off; Awake/preference
true; Sleep allowed/true; Sleep allowed/false; Sleep allowed/true; Awake/true;
Awake/false; explicit final clear. The user's confirmation also covers the
requested icon, tooltip, checkmark, left-click/menu, and right-click-dismissal
checks. The artifact was unchanged; earlier automated results were reused.

### Completed native procedure

In normal PowerShell launch the artifact above. In a separate Administrator
PowerShell, run `powercfg /requests` before launch, after each state below, and
after Exit. Check the icon, tooltip, and menu alongside the external requests.

| Active | Keep display on preference | Expected RedEye requests and UI |
|---|---|---|
| Yes | Off (launch default) | SYSTEM only; bright icon, Awake tooltip; Stay awake checked |
| Yes | On | SYSTEM and DISPLAY; both menu options checked |
| No | On | Neither request; dark icon, Sleep allowed tooltip; display option remains checked |
| No | Off | Neither request; both menu options unchecked |

While inactive, change the display preference and then enable again; verify
the selected preference takes effect only when active. Repeat left-click and
Stay awake menu toggles; expect one transition per action and consistent
requests/visuals. Open and dismiss the right-click menu; expect no transition.
Choose Exit; expect icon removal, explicit-clear output, process exit, and no
RedEye request. Retain the labeled user observations/diagnostics.

Windows 10, monitor/DPI changes, Explorer restart, lock/explicit sleep, forced
termination, long-duration idle behavior, and release-only behavior remain
untested. Settings, startup registration, and duplicate launch are later gates.

## Milestone 4 — 2026-11-08

Status: milestone 4 core acceptance complete on the tested Windows 11
environment. Independent final review found no defects; automated checks
passed. The user confirmed all six requested native procedure steps. The
validator did not write settings/startup registration, launch processes, or
send UI input while the user tested.

Environment: Windows 11 Home build 26300 x64 / Rust 1.99.1 MSVC, as above.
Artifact: frozen working-tree milestone 4, `target/debug/redeye.exe`, SHA256
`A3B85D1C2514B098C45A36B5A131D1EA63FC0CB979FFAD2223AF8B6091188909`.
The hash matched before/after checks. `portable-test/RedEye test.exe` is a
byte-identical standalone debug copy outside Cargo output, with a space in its
filename for the startup-quoting test; it has not been launched by the validator.
This test copy is not a release artifact. Existing icon evidence and unchanged
crate versions/lockfile (`windows-sys 0.61.2`, `windows-link 0.2.1`) are reused.

| Check | Actual result | Status |
|---|---|---|
| `cargo fmt --check` | Exit 0 | Pass |
| `cargo check --locked` | Exit 0 | Pass |
| `cargo clippy --locked --all-targets -- -D warnings` | Exit 0, no lint findings | Pass |
| `cargo test --locked` | Seven tests passed, including display/startup parsing and quoted stable/disposable-path rules | Pass |
| `cargo build --locked` | Exit 0; expected artifact hash unchanged | Pass |
| Copied executable with space in filename | Copy hash equals the debug artifact hash above | Pass: preparation only |
| README usage/settings/startup/removal and local links | Matches saved display preference, active restart, actual HKCU registration, stable-path refusal, session instance behavior; all local links exist; `git diff --check` clean | Pass |
| Duplicate launch | User confirmed one existing icon/instance remains unaffected and the new launch exits without another icon | Pass: user confirmation |
| Preference/restart | User confirmed display-on preference saved; exiting while inactive then restarting starts active with SYSTEM and DISPLAY | Pass: user confirmation |
| Malformed setting | User confirmed DWORD 2 produces a warning and display-off default; changing the option saves valid data | Pass: user confirmation |
| Cargo-output startup refusal | User confirmed notice and unchecked/absent registration | Pass: user confirmation |
| Stable copied startup registration | User confirmed quoted copied path with space, enable/readback/menu agreement, then disable/removal | Pass: user confirmation |
| Normal Exit | User confirmed no RedEye requests after Exit | Pass: user confirmation |

The native results above come from the user's summary confirmation, not full
diagnostic transcripts. The user noted that launching `& $exe` while RedEye was
already running neither exits the existing instance nor creates another; this
is the intended duplicate behavior. The existing owner remains, and the newly
launched duplicate exits. The unchanged artifact hash allowed reuse of the
automated checks without rebuilding or repeating tests.

The lead's pre-implementation external HKCU baseline found no RedEye Run value.
Real sign-in launch, Explorer restart recovery, cross-session isolation, and
Windows 10 coverage are **not run**. Focused settings tests exercise parsing and
command construction without writing the user's registry. They do not prove
actual sign-in execution or Explorer recovery.

### Native procedure and remaining release checks

User-operated milestone 4 checks: launch normally and launch a second copy;
expect one process/icon and a clean duplicate exit. Change Keep display on,
exit, and restart; expect restored display preference and active state on each
launch, regardless of prior awake state. With the application closed, retain
the original RedEye display value before testing malformed data; expect a
clear notice and safe display-off default, then restore/save valid data.

From Cargo output, enabling Launch at sign-in must show the stable-location
notice and leave registration absent. From
`C:\Users\tbrow\dev\RedEye\portable-test\RedEye test.exe`, enable the option;
inspect `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` value `RedEye`
and expect the fully quoted path, with the menu checked on reopening. Disable
and expect the value absent and menu unchecked. Do not change unrelated values.
Capture `powercfg /requests` for saved display-on restart and after normal Exit.

Release-readiness checks still requiring deliberate user action:

- **Real sign-in (not run):** from a stable final executable enable Launch at
  sign-in, confirm the quoted HKCU value, save work, sign out/in, and verify one
  active icon plus expected SYSTEM/DISPLAY requests and restored preference.
  Disable startup afterward if desired. Startup Apps policy may also affect
  execution; record the actual result, not only the Run value.
- **Explorer restart (not run):** with RedEye active, use Task Manager to
  restart Windows Explorer when safe; verify icon returns, menu/toggle works,
  and power requests agree with effective state before/after. Do not count
  compilation of the TaskbarCreated handler as evidence.
- **Cross-session isolation (not run):** in another interactive user/session,
  launch RedEye independently; verify its icon/request is usable and that a
  duplicate within each session exits. Exit both copies afterward. No session
  change was performed by the validator.
- **Windows 10 (not run):** repeat the native lifecycle/state/startup matrix on
  an identified Windows 10 build and record the OS/artifact and outcomes.

## Milestone 5 — 2026-11-08

Status: final cumulative independent source review found no defects. Required
checks/release build and initial portable packaging passed. The user confirmed
the copied executable's native portable procedure below. A subsequent bounded
executable-icon resource change is in progress; this section identifies the
historical pre-resource artifact, not the final package. The validator did not
launch or control the application, send UI input, or write startup/settings
registration during this validation.

Environment: Windows 11 Home build 26300 x64 / Rust 1.99.1 native MSVC.
Release artifact: `target/release/redeye.exe`, 407,040 bytes, SHA256
`11AE78034EAB953983A4EDA2CA27A244BA0DCFE72617A1474B42FCA9D7B4CD68`.
The copied `dist/RedEye-v0.1.0-windows-x64/RedEye.exe` has the same hash.
The hash was unchanged before/after checks and release build.

| Check | Actual result | Status |
|---|---|---|
| `cargo fmt --check` | Exit 0 | Pass |
| `cargo check --locked` | Exit 0 | Pass |
| `cargo clippy --locked --all-targets -- -D warnings` | Exit 0, no lint findings | Pass |
| `cargo test --locked` | Seven tests passed | Pass |
| `cargo build --release --locked` | Exit 0; expected optimized artifact/hash | Pass |
| PE header | Machine `0x8664` x64, PE32+ `0x20B`, subsystem 2 Windows GUI; independently confirmed by `dumpbin /headers` | Pass |
| `dumpbin /dependents` | Only Windows system imports listed below; no VCRUNTIME, MSVCP, UCRT, or other separately shipped runtime DLL | Pass |
| Source dependency lock | App plus `windows-sys 0.61.2` and `windows-link 0.2.1`; no package/version change | Pass |
| Portable package contents | Exe, standalone README.txt, SHA256SUMS.txt only; no loose icons, DLLs, Cargo, or source requirements at runtime | Pass |
| Copied release launch, no console/elevation, menu/state/Exit and native requests | User confirmed the requested procedure, including the four-state contract and cleanup; no raw diagnostic transcript supplied | Pass: user confirmation for this artifact |

The checked-in `.cargo/config.toml` sets `+crt-static` for native x64 MSVC.
`dumpbin` listed `KERNEL32.dll`, `user32.dll`, `shell32.dll`, `advapi32.dll`,
`api-ms-win-core-synch-l1-2-0.dll`, and `ntdll.dll`. Windows supplies these system
libraries. Header/import inspection supports portability; the user separately
confirmed runtime behavior. The package has not been published, tagged,
uploaded, or code-signed. ZIP/checksum finalization is paused for the requested
original executable icon resource; a new artifact hash and resource/native
smoke checks will be recorded after that change is reviewed.

The native core results from milestones 1–4 and unchanged asset checks are
reused for the unchanged functional paths. Release changes select a GUI
subsystem, static CRT, and system-DPI initialization; their native run was
confirmed separately by the user. Historical debug hashes identify earlier
tests, not this release executable.

### Completed portable native procedure for the pre-resource artifact

With earlier RedEye copies exited, double-click the copied `RedEye.exe` from
the dist folder in Explorer. Expect one tray icon, no console/main window, and
no elevation prompt. Verify tooltip, left-click/menu awake toggles, the display
option, and right-click dismissal. In a separate Administrator PowerShell,
capture `powercfg /requests` while active/display off, active/display on,
inactive with both preferences, and after Exit. Expect the same four-state
contract and no RedEye request after normal Exit. Check the menu remains
usable from the copied executable without Cargo or loose assets.

The user confirmed this procedure for SHA256
`11AE78034EAB953983A4EDA2CA27A244BA0DCFE72617A1474B42FCA9D7B4CD68`.
This is user summary confirmation, not an agent-run UI test or raw powercfg
transcript. The subsequent resource-only build, native icon extraction, and
reuse of unchanged functional evidence are detailed in the final section
below. No application launch is claimed for the resource-only update.

Actual sign-in launch, Explorer restart, cross-session isolation, Windows 10,
DPI/monitor changes, lock/explicit sleep, forced termination, and long-duration
resource behavior remain **not run**. These limits prevent a claim of full
Windows 10/11 release validation. Sign-in, Explorer, cross-session, and Windows
10 follow-up steps are recorded in milestone 4. For DPI, change Windows display
scaling or move between different-DPI monitors when safe, restart RedEye, and
check both states/readability and menu. For lock/sleep, save work, lock/unlock
and explicitly sleep/resume, verify Windows behavior is respected, then check
RedEye's state/requests and Exit cleanup. For forced termination, end only a
known RedEye test process and verify its requests disappear; note shell icon
cleanup delay. Observe CPU/memory after settling and during a longer idle
period without injecting input or keep-awake polling, recording duration and
environment rather than treating the earlier 3-second debug sample as coverage.

## Final executable icon and package — 2026-11-08

Status: the original active eye is now embedded as the native executable icon.
Independent final review found no defects. Required checks, resource/native
extraction verification, and final packaging passed. Runtime source is unchanged
from the user-tested portable GUI release (`0ce780f`); its functional native
evidence is reused. Actual Explorer cache appearance of the resource-updated
executable remains a manual refresh check, not an additional claimed UI pass.
The remaining release-readiness limitations above still apply; this is a
portable release candidate, not full Windows 10/11 validation.

Final `target/release/redeye.exe` and packaged `RedEye.exe`: 474,624 bytes,
SHA256 `2B6555F4D449CA94EE2AC71B9FAD756DBB8B49CFCDB4DBAF8CE9564FB5CD272D`.
This supersedes the pre-resource executable and provisional ZIP. The package
contains exactly these three files under `RedEye-v0.1.0-windows-x64`:

- `RedEye.exe`, matching the final release hash above.
- `README.txt`, 4,945 bytes, standalone installation/use/build/removal and
  validation-status instructions; no source-local links to missing files.
- `SHA256SUMS.txt`, hashes of the executable and README.

Final archive: `dist/RedEye-v0.1.0-windows-x64.zip`, 322,557 bytes,
SHA256 `6C0E1FBACFA91557EDD5D36B4AFF8289E6185E676F5CFEE1C42C1200C8C69A4B`.
The validator reopened the ZIP and hashed each payload; all three match the
package directory. No loose icons, DLLs, Cargo, or SDK files are runtime content.
Nothing was published, tagged, uploaded, or code-signed.

| Check | Actual result | Status |
|---|---|---|
| `cargo fmt --check` | Exit 0 | Pass |
| `cargo check --locked` | Exit 0 | Pass |
| `cargo clippy --locked --all-targets -- -D warnings` | Exit 0, no lint findings | Pass |
| `cargo test --locked` | Seven tests passed | Pass |
| `cargo build --release --locked` | Exit 0; expected hash unchanged | Pass |
| SDK resource compiler discovery | Build log selects SDK 10.0.26100.0 x64 `rc.exe`, although `rc.exe` is not on the shell PATH; no PATH changes | Pass |
| Final PE resource directory | One `RT_GROUP_ICON` ID 1 and eight `RT_ICON` resources, sizes 16/20/24/32/48/64/128/256; 32-bit RGBA PNG bytes exactly equal original active ICO frames; no `RT_MANIFEST` resource | Pass |
| PE header/imports | Still x64 PE32+, Windows GUI subsystem 2; same six Windows system imports as above, no separate CRT DLL | Pass |
| Native Windows icon extraction | `ExtractIconExW` extracted small 16px and large 32px icons; resource-only mapping plus `LoadImageW` extracted 64px; original eye visually confirmed | Pass: read-only native API evidence |
| Runtime source/dependency scope | No `src` diff from `0ce780f`; runtime crates remain `windows-sys 0.61.2` and `windows-link 0.2.1`; build-only additions are `winresource 0.1.34` with default features disabled and `version_check 0.9.5` | Pass |
| Package/docs | Final copied hash, three ZIP payloads, checksums, current SDK/build/icon instructions, and local source-document links verified | Pass |

PE parsing read file bytes only. Native icon checks used `ExtractIconExW` and
`LoadLibraryExW` with `LOAD_LIBRARY_AS_DATAFILE | LOAD_LIBRARY_AS_IMAGE_RESOURCE`,
then loaded icon resource ID 1. These operations did not execute the application
entry point, launch a process, control UI, or write settings/registration. Owned
HICONs and the resource mapping were explicitly released. The preview at
`portable-test/embedded-exe-icon.png` shows Shell-extracted 16/32px icons and the
64px resource image; it is validation evidence, not runtime package content.

For the remaining Explorer cache check, refresh the folder or inspect a copy
under a fresh executable filename if Windows retains an old generic icon.
Confirm the original active eye appears. The read-only native extraction and
exact frame comparison establish that Windows can obtain the intended icon;
no additional user runtime claim is inferred for this last resource-only build.

## CI and main-branch rules — 2026-11-08

Local checkpoint before the first hosted run: independent review found no
defects in the final CI/ruleset change on
`ci/protect-main`. Local policy, checksum-script, and documentation checks passed.
The lead applied the reviewed rules and supplied server readback below. The
feature branch/PR and actual hosted Windows workflow have not yet been validated
at that checkpoint. The hosted results below supersede that initial execution
status; local file checks alone do not establish a successful hosted run.

The validator changed only this evidence document and temporary validation
files under `portable-test/ci-validation-tools`. No workflow, rule JSON,
CODEOWNERS, application code, credentials, remote settings, commits, or pushes
were changed by the validator. The previously passing seven Rust tests and
release-build evidence are reused for unchanged application/build sources;
the local release SHA256 remains
`2B6555F4D449CA94EE2AC71B9FAD756DBB8B49CFCDB4DBAF8CE9564FB5CD272D`.
No GUI or native power tests were repeated for CI configuration.

| Check | Actual result | Status |
|---|---|---|
| Rules JSON parsing and exact policy checks | Both JSON documents parse; target only `refs/heads/main`; active; layered requirements match below | Pass |
| CODEOWNERS | Sole noncomment entry is `* @tbrownhe`, including CI/ruleset definitions | Pass |
| Workflow inspection | PRs to main, main pushes, manual dispatch; read-only contents; no privileged PR event/secrets; no persisted checkout credentials; pinned first-party actions/toolchain; expected runner/check/artifact | Pass: source inspection |
| PowerShell checksum script | Parsed without AST errors; executed against a temporary copied release fixture; lowercase SHA256 plus `redeye.exe` output matches; both artifact paths exist | Pass |
| Local source Markdown links and whitespace | All 35 local links resolve; `git diff --check` clean | Pass |
| Local workflow YAML/schema/expression lint | Official actionlint 1.7.12 Windows/amd64, checksum verified before execution; `actionlint .github/workflows/ci.yml` exit 0 with no findings | Pass |
| Actual GitHub parser, Windows CI job, and artifact | Successful hosted execution and independent artifact verification recorded below | Pass: hosted evidence |

Workflow inspected: runner `windows-2025-vs2026`, job/check `Windows CI`,
20-minute timeout, `RUSTUP_TOOLCHAIN=1.99.1-x86_64-pc-windows-msvc`, required
Rust checks and release build. Checkout is pinned to
`3d3c42e5aac5ba805825da76410c181273ba90b1`; upload-artifact to
`cf430e030ddbb5b0abf93d22962f4752f3646cd9`. Superseded runs cancel only for the
same PR; main/manual runs have unique run-ID groups. Successful artifact
`RedEye-windows-x64` contains only the executable/checksum and expires after
seven days. Source inspection does not substitute for execution on that runner.

Temporary actionlint came from the
[official v1.7.12 release](https://github.com/rhysd/actionlint/releases/tag/v1.7.12).
Direct downloads returned HTTP 504; the official GitHub asset API fallback
succeeded. The archive SHA256
`6E7241B51E6817EA6A047693D8E6FED13B31819C9A0DD6C5A726E1592D22F6E9`
matched GitHub's published asset digest before extraction/execution. Its
reported version was 1.7.12. No permanent installation, PATH change, project
dependency, workflow change, or linter suppression was needed.

The lead supplied authenticated GitHub API readback after applying the reviewed
JSON through API version `2026-03-10`:

- Active [main-integrity ruleset](https://github.com/tbrownhe/RedEye/rules/26833257)
  ID `26833257`, no bypass actors. Requires a PR, resolved conversations, strict
  up-to-date `Windows CI` from GitHub Actions integration `15368`; blocks deletion
  and non-fast-forward changes.
- Active [owner-review ruleset](https://github.com/tbrownhe/RedEye/rules/26833258)
  ID `26833258`, one approval, CODEOWNER approval, stale approval dismissal.
  Only User `97360755` (`@tbrownhe`) has the user-approved `pull_request` review
  bypass. It does not bypass the independent integrity/CI ruleset.
- The effective main-branch endpoint returned five rules: deletion,
  non-fast-forward, integrity PR/conversation requirements, required status,
  and owner-review PR requirements. No weakening or temporary exception was
  reported. This is lead-supplied server readback, not a validator remote write.

The bootstrap PR must merge only after a successful check on its final head,
using the explicitly authorized owner review exception through the PR. A docs
evidence update receives its own required CI run before merge. The bootstrap
PR installs CODEOWNERS on the base branch for future contribution reviews.
Existing unperformed native release checks remain unperformed; CI does not
prove tray behavior, power requests, sign-in, or Windows 10 coverage.

### First hosted PR run and artifact

The lead supplied completed GitHub API evidence for
[PR #1](https://github.com/tbrownhe/RedEye/pull/1), authored by `@tbrownhe`, head
`73c2cc33188c003064371dfc42efd8af4e02fc7d`:
[run 42572845456](https://github.com/tbrownhe/RedEye/actions/runs/42572845456)
was triggered by `pull_request` and succeeded. All 13 recorded steps succeeded,
including formatting, check, Clippy, tests, release build, checksum, and upload.
Check run `129013923445` is named `Windows CI`, belongs to GitHub Actions app
`15368`, concluded success, and matches that head. Thus actual GitHub workflow
parsing, runner selection, and the required check identity are verified, not
only local YAML lint.

The validator independently inspected bounded entries from the downloaded job
logs. They identify Windows Server 2025 Datacenter `10.0.26100`, hosted image
`windows-2025-vs2026` version `20261103.298.1`, runner `2.338.2`, pinned
`1.99.1-x86_64-pc-windows-msvc`, and rustc `1.99.1 (58cf8f9c3)`. Seven tests
passed and the optimized build completed. The successful resource build and
embedded frames establish SDK/resource-compiler availability; these normal
logs do not report its exact SDK version, so none is claimed for this runner.
The setup log confirms Contents/Metadata read-only token permissions and the
two expected pinned action SHAs.

The lead downloaded
[artifact 13653687586](https://github.com/tbrownhe/RedEye/actions/runs/42572845456/artifacts/13653687586)
(`RedEye-windows-x64`, seven-day retention, not expired) and logs without
forwarding credentials to the download redirect. The validator read these
local archives without reading credentials or making remote writes:

| Artifact check | Result |
|---|---|
| Downloaded ZIP | 323,892 bytes; SHA256 `A843B425CE003F11F5278C705EE33E690D76A316401B30C0E51E41BE4AC04990`, matching upload log and supplied GitHub digest |
| Contents/bounds | Exactly `redeye.exe` and `SHA256SUMS.txt`; bounded sizes, CRC reads passed; extraction writes only fixed filenames under the validation scratch directory |
| Executable checksum | 474,624 bytes; SHA256 `F7B6E6FB2E4E7525264A42A6076C12C8352D0EBE7A8B18C70D506B16220DD929`, matching the artifact checksum file |
| PE architecture/subsystem | x64 PE32+, Windows GUI subsystem 2 |
| Imports | Same six Windows system DLLs as the local portable candidate; no separate CRT runtime DLL |
| Icon resources | Group ID 1 plus eight 32-bit original-active PNG frames 16 through 256 pixels; every payload byte-identical to source ICO; no manifest resource |
| Windows icon extraction | Native read-only `ExtractIconExW` yielded 16px/32px icons; handles released; no executable entry point or application process was run |

The hosted executable hash differs from the locally built portable candidate;
each is identified by its own checksum and evidence. No claim of byte-for-byte
cross-environment reproducibility or hosted GUI/power behavior is made.

The lead's separate API readback confirmed the existing fork workflow approval
policy remains `first_time_contributors`, default workflow token permissions
are read-only, and workflows cannot approve PRs. Both active ruleset IDs and the
five effective rules remained unchanged. PR mergeability reported true but
`mergeable_state` remained `blocked` even after CI success; the owner-review
bypass/actual merge had not yet been exercised at this evidence checkpoint.
At that checkpoint the base branch did not yet contain CODEOWNERS.

This evidence update changes documentation only. Each later commit must pass
its own required CI; the first green run does not substitute for checks on a
later documentation commit. Final bootstrap merge state is recorded on
[PR #1](https://github.com/tbrownhe/RedEye/pull/1), and subsequent PR/main runs
are recorded by [CI](https://github.com/tbrownhe/RedEye/actions/workflows/ci.yml).
All previously listed unperformed native release checks remain unperformed.

## Tagged draft-release automation — 2026-11-08 local checkpoint

Status: separate final review found no defects; local validation passed on
`ci/tagged-draft-releases`. Protected PR/main CI, manual preview, owner RC tag,
draft creation, and uploaded release assets remain pending. The validator made
no remote writes, tags, releases, app launches, GUI actions, or registry changes.

The version is `0.1.0-rc.1`; the four dependency packages and runtime source/build
configuration are unchanged. The RC executable has changed version metadata,
with no runtime behavior change. Its exact GUI runtime has not been tested;
prior functional evidence is reused and outstanding Windows checks remain open.

| Local check | Result |
|---|---|
| Format/check/Clippy | Required locked commands exit 0 |
| Tests/release build | Seven tests pass; locked optimized build succeeds |
| Product/File version | Both exactly `0.1.0-rc.1` through Windows version-resource inspection |
| Workflow/script validation | Previously checksum-verified actionlint 1.7.12 accepts both workflows; AST parsing accepts both helpers and the entire inline publisher |
| Release gate | Fifteen independent mocked boundary cases pass: valid tag/manual-main succeed; wrong tag/ref/event, feature manual, nonancestor, wrong CI SHA/event/path/conclusion/status/branch/repository, invalid SemVer reject without success outputs |
| Actual packager | Valid RC package succeeds; leading-zero SemVer, different version, malformed commit, workspace-root output, and traversal outside allowed output roots reject |
| Publisher verification prefix | Valid package passes identity/checksum/ZIP checks; corrupted copied exe rejects before remote operations |
| ZIP/integrity | Exactly exe/README/internal sums; all internal/external hashes match; standalone/ZIP exe bytes agree; version/commit placeholders resolved |
| PE inspection | x64 GUI subsystem 2, six Windows system imports/no separate CRT; group 1/eight original-active 32-bit frames byte-identical to ICO; no manifest resource |
| Tag policy | Exact local active `refs/tags/v*` creation/update/deletion policy; only owner User `97360755` has always bypass; main policies unchanged |
| Documentation/scope | All 47 local Markdown links resolve; whitespace clean; no runtime source, main rules, CODEOWNERS, or cleanup-setting changes |

Local RC executable: 474,624 bytes, SHA256
`D6BB4B049F2DF9EDF7B2852C5EC4BB53B803608681D9CDD6B6B3A8AD42935FBE`.
The validator's scratch ZIP SHA256 is
`339A65EAF0E673A0A69946E17919222D6C346040F8D859B16CC19DCF29E73702`.
Its README stamps base HEAD `900b29ea5258af336520cc4f150693b5221b0c5d`
because validation preceded the release-change commit. This is a working-tree
fixture, not evidence of an approved RC build from that base commit. Each
packaging run's hashes are checked independently; ZIP hashes can differ.

Mocks executed the real gate with in-process Git/GitHub/Cargo functions that
cannot access the network or mutate refs, including checking the exact main-push
CI endpoint. Publisher execution stopped before tag/release API calls; its full
script was only parsed. Actual packaging used a process-only execution-policy
bypass and wrote ignored fixtures. No permanent PATH or policy changed.

The lead supplied remote readback for active
[release-tags ruleset 26835818](https://github.com/tbrownhe/RedEye/rules/26835818):
tag target, `refs/tags/v*`, creation/update/deletion restrictions, only owner
User `97360755` always bypass. Both main rulesets remained byte-equivalent,
their five effective rules unchanged, and automatic merged-branch cleanup
remained enabled. This is lead-supplied evidence, not a validator API mutation.

Remote completion requires successful exact-head PR CI, merge/main-push CI,
manual preview with no tag/release, then the authorized exact owner RC tag and
an unpublished prerelease draft with three verified assets/checksum layers.
Tag/main SHA and successful main-CI evidence must agree. The draft stays
unpublished until the owner manually chooses Publish. Local mocks/builds prove
neither these remote outcomes nor additional native Windows coverage.
