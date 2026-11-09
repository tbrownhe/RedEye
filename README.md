# RedEye

A small Windows 10/11 x64 tray utility for preventing automatic idle sleep.

The v0.1 implementation is available as a portable release candidate. Windows
11 native core behavior has been checked; Windows 10 and the remaining manual
release checks are unverified. See [validation evidence](docs/VALIDATION.md)
for the exact artifacts and limitations, and [the workflow](docs/WORKFLOW.md)
for implementation decisions.

## Product scope

RedEye supports long-running computational work, including autonomous coding
sessions, through native Windows power requests instead of periodic NumLock
input simulation. Keep the utility compact, dependable, low in CPU/memory
use, and understandable to an engineer learning Rust. Correctness and
readability take priority over feature count.

The v0.1 product is a native notification-area utility with no visible main
window, service, elevation, embedded browser, telemetry, runtime networking,
or automatic updates. Keep-awake behavior must not poll or simulate input.
Respect locking, explicit sleep/shutdown, organizational policies, and remote
session inactivity. Avoid large UI frameworks such as Tauri, Electron, and
PySide, unnecessary async runtimes, and databases.

Out of scope for v0.1: timed or scheduled awake sessions, global hotkeys,
automatic process/task detection, desktop widgets or floating overlays, a
full settings window, integrations (including Codex-specific integration),
remote control, and cloud synchronization.

Deliver a working Rust project, portable Windows executable with embedded
original active/inactive icons, reproducible build instructions, usage and
removal documentation, and concise explanations of the relevant Rust and
Windows decisions. [Icon direction](docs/ICONS.md),
[milestone gates](docs/WORKFLOW.md), and [test evidence](docs/VALIDATION.md)
complete the product baseline.

## Install and run

Copy `RedEye.exe` from the portable package to a stable folder, such as
`%LOCALAPPDATA%\RedEye`, and launch it. No installer, administrator privileges,
Cargo, or external icon files are required. The release build has no console
or visible main window. Nothing is registered for sign-in until you enable it.

## Build from source

Use stable Rust with the native Windows x64 MSVC toolchain, C++ build tools,
and Windows SDK resource compiler (`rc.exe`).
Rust/Cargo 1.99.1 was verified for this project:

```powershell
cargo build --release --locked
& .\target\release\redeye.exe
```

The checked-in target configuration links the CRT statically, avoiding a
separate Visual C++ runtime installation. Copy the executable to a stable
folder before enabling sign-in launch. For console diagnostics during
development, use `cargo build --locked` and `target\debug\redeye.exe`.

## Use

RedEye starts active, with the display allowed off by default; a saved display
preference is restored. Its original red eye
icon appears in the notification area, possibly in the overflow. Left-click
toggles awake state. Right-click for a native menu:

- **Stay awake** toggles automatic sleep prevention.
- **Keep display on** also prevents display shutdown while awake. The checked
  preference remains available while inactive, without holding a power request.
- **Launch at sign-in** opts into per-user Windows startup registration.
- **Exit** removes the icon, explicitly clears the power request, and exits.

Bright/dark eye icons, tooltips, and checkmarks show the effective state after
Windows accepts a change. Display preference is saved per user; awake state is
not saved, so every launch starts active. A second launch in the same user
session exits without another icon; it does not stop the existing instance.
Use **Exit** for explicit cleanup. Forced termination relies on Windows
releasing the thread-owned request; Explorer may briefly retain a stale icon.

RedEye runs without elevation. Windows may require an Administrator terminal
for the separate `powercfg /requests` diagnostic. It does not prevent locking,
explicit sleep/shutdown, or organizational policy enforcement.

The icon uses system DPI at launch. Restart after changing display scaling
for the sharpest size. Per-monitor scaling changes and multiple DPI settings
have not been validated.

## Sign-in launch and settings

Copy the executable to a stable folder, such as `%LOCALAPPDATA%\RedEye`, and
launch that copy before checking **Launch at sign-in**. RedEye refuses startup
registration from known Cargo output or temporary directories. Paths are
quoted and must fit Windows Run's 260-character command limit.

Only `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`'s `RedEye` value is
changed. The menu rereads its actual registration whenever opened. A nonempty
registration to another copy also shows checked, allowing stale entries to be
disabled. After moving the executable, disable startup and enable it from the
new location. Windows startup controls and organizational policy can still
disable or delay sign-in launch; RedEye does not override them.

The display preference is DWORD `KeepDisplayOn` under `HKCU\Software\RedEye`.
Missing data defaults off; malformed data produces a warning and defaults off
for that launch. A failed save reports that the current preference applies
only to the current run. No active-state preference is stored.

To remove RedEye, disable **Launch at sign-in**, choose **Exit**, and delete the
copied executable. Optionally remove only the `KeepDisplayOn` value from the
settings key. No service, scheduled task, or machine-wide setting is installed.

## Check

```powershell
cargo fmt --check
cargo check --locked
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

## Code layout

| File | Responsibility |
|---|---|
| `src/main.rs` | Initialization, effective state and display preference, main-thread event coordination, and graceful shutdown |
| `src/power.rs` | Small safe `PowerRequest` interface around thread-owned `SetThreadExecutionState`; independent of UI code |
| `src/tray.rs` | Hidden native window, blocking event dispatch, icon/menu/tooltip updates, and owned UI resources |
| `src/settings.rs` | Per-user display preference and actual opt-in startup registration |
| `src/instance.rs` | Session/user single-instance guard |
| `build.rs` | Build-time executable icon resource embedding |

Keep ownership, borrowing, unsafe FFI boundaries, native event-loop behavior,
and explicit cleanup explanations tied to the code being changed. Avoid
abstractions or modules without a current consumer. Single-instance protection
must stay lightweight; do not introduce IPC or a service merely to enforce it.

`windows-sys` 0.61.2 supplies native tray bindings, with `windows-link` as its
only supporting crate. `power.rs` wraps one scalar Kernel32 call. Its owner
stays on the main thread alongside the blocking native event loop. State
changes are committed only after Windows accepts them. Normal shutdown
explicitly handles cleanup errors; destructors provide fallback handle
cleanup. Tests cover the flag matrix, failed transitions, and icon bounds;
external Windows observations are also required. Icons are embedded, with
[reproducible drawing source](docs/ICONS.md).

The build also uses `winresource` 0.1.34 with default features disabled and
`version_check` to embed the active eye as the executable's Explorer/shortcut
icon. These are build-only dependencies; both tray states remain embedded.
