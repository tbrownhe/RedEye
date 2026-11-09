# RedEye

A small Windows 10/11 utility for preventing automatic idle sleep.

Development is at **milestone 4: persistence and lifecycle**. Portable release
packaging is next. See
[the workflow](docs/WORKFLOW.md) for status and
[validation evidence](docs/VALIDATION.md) for checks and remaining native tests.

## Build and run

Use stable Rust with the native Windows x64 MSVC toolchain and C++ build tools:

```powershell
cargo build --locked
& .\target\debug\redeye.exe
```

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
session exits without another icon. The debug build still has a console.
Closing it terminates the process
and relies on Windows releasing the thread-owned request.

RedEye runs without elevation. Windows may require an Administrator terminal
for the separate `powercfg /requests` diagnostic. It does not prevent locking,
explicit sleep/shutdown, or organizational policy enforcement.

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

`windows-sys` 0.61.2 supplies native tray bindings, with `windows-link` as its
only supporting crate. `power.rs` wraps one scalar Kernel32 call. Its owner
stays on the main thread alongside the blocking native event loop. State
changes are committed only after Windows accepts them. Normal shutdown
explicitly handles cleanup errors; destructors provide fallback handle
cleanup. Tests cover the flag matrix, failed transitions, and icon bounds;
external Windows observations are also required. Icons are embedded, with
[reproducible drawing source](docs/ICONS.md).
