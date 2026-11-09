# RedEye

A small Windows 10/11 utility for preventing automatic idle sleep.

Development currently stops at **milestone 1: console power control**. The
notification-area interface, preferences, and sign-in launch are later
milestones. See [the workflow](docs/WORKFLOW.md) for status and
[validation evidence](docs/VALIDATION.md) for checks and remaining native tests.

## Build and run

Use stable Rust with the native Windows x64 MSVC toolchain and C++ build tools:

```powershell
cargo build --locked
& .\target\debug\redeye.exe
```

RedEye starts active and allows the display to turn off. Enter `off` to allow
automatic sleep, `on` to prevent it again, `status` to print the last
successfully applied state, or `quit` to clear the request explicitly and exit.
EOF also clears and exits. Use `quit` for normal shutdown; Ctrl+C or closing
the console terminates the process and relies on Windows releasing the
thread-owned request.

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

The application has no third-party dependencies. `power.rs` wraps one scalar
Kernel32 call. Its owner stays on the main thread, and state changes are
committed only after Windows accepts them. Main explicitly handles cleanup
errors rather than hiding them in a destructor. Tests cover the flag matrix
and failed transitions; external Windows observations are also required.
