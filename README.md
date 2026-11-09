# RedEye

A small Windows 10/11 utility for preventing automatic idle sleep.

Development is at **milestone 2: native tray integration**. Interactive
toggling, preferences, and sign-in launch are later milestones. See
[the workflow](docs/WORKFLOW.md) for status and
[validation evidence](docs/VALIDATION.md) for checks and remaining native tests.

## Build and run

Use stable Rust with the native Windows x64 MSVC toolchain and C++ build tools:

```powershell
cargo build --locked
& .\target\debug\redeye.exe
```

RedEye starts active and allows the display to turn off. Its original red eye
icon appears in the notification area, possibly in the overflow. Right-click
it for a native menu and choose **Exit** to remove the icon, explicitly clear
the power request, and exit. Left-click currently prints a diagnostic and
leaves power state unchanged; toggling belongs to milestone 3. The debug build
still has a console. Closing that console terminates the process and relies
on Windows releasing its thread-owned request.

RedEye runs without elevation. Windows may require an Administrator terminal
for the separate `powercfg /requests` diagnostic. It does not prevent locking,
explicit sleep/shutdown, or organizational policy enforcement.

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
