# RedEye agent instructions

## Read first

Read [handover.md](handover.md) for the product requirements and
[docs/WORKFLOW.md](docs/WORKFLOW.md) for roles, milestone gates, and validation.
Inspect the current files and working-tree changes before editing. Preserve
user work. Use this uppercase `AGENTS.md` as the repository instruction entry
point; do not create a second lowercase copy.

The goal is a compact, dependable Windows 10/11 tray utility that prevents
automatic idle sleep, with an implementation understandable to an engineer
learning Rust. Simplicity and correctness take priority over feature count.

## Scope and working style

- Follow the user's current task. Workflow setup does not authorize building
  the entire application. During implementation, complete a small, testable
  milestone before expanding scope.
- Treat the handover as the product baseline. Record material technical
  decisions and their reasons in `docs/WORKFLOW.md`; do not silently change
  required behavior. Ask only when a product choice materially changes scope
  or user expectations. Resolve routine engineering choices independently.
- Use the five roles in the workflow as responsibilities, not five permanent
  processes. One agent can perform them sequentially. Delegate only when the
  user explicitly requests delegation or applicable task instructions require
  it. When delegated, give each task a bounded deliverable and clear file
  ownership; the lead remains responsible for integration and verification.
- The main agent is the lead/integrator. Reusable project specialists live in
  `.codex/agents/`: `windows_engineer`, `ux_designer`, `correctness_reviewer`,
  and `release_validator`. Use their definitions for delegated tasks; see the
  workflow for invocation examples and role boundaries. Definitions do not
  automatically launch agents or grant separate permissions.
- Before choosing dependencies, verify current stable APIs and compatibility
  using official documentation and crate sources. Record the chosen versions,
  required features, Windows support, and event-loop/threading constraints.
  Prefer the smallest working solution; do not select a framework by habit.
- Explain meaningful Rust decisions briefly as they arise: types, ownership,
  borrowing, unsafe boundaries, event-loop behavior, and resource cleanup.
  Keep learning notes tied to actual code, not a separate textbook.
- Close each task with what changed, what was checked, what remains unverified,
  and the next concrete step. Never label an unperformed check as passed.

## Product guardrails

- Native notification-area UI, no visible main window in the release build.
  A hidden native window for message dispatch is acceptable.
- No application elevation, service, embedded browser, async runtime without
  a concrete need, telemetry, runtime networking, or automatic updates.
- No artificial keyboard/mouse input or polling to keep the machine awake.
- Do not interfere with locking, explicit sleep/shutdown, organizational
  policies, or remote-session inactivity behavior.
- Keep the handover's v0.1 exclusions out of scope, including timers, hotkeys,
  process detection, full settings windows, and integrations.
- Start active on manual and sign-in launch. Default to allowing the display
  to turn off. Sign-in launch is opt-in. Do not persist the active state.
- Left-click toggles awake state; right-click opens a native menu. Icon,
  tooltip, and checkmarks must accurately represent the effective state.
- Use `icon_idea.jpg` as a visual reference, not a finished tray asset. Deliver
  original active/inactive icons readable at 16x16 and suitable for high DPI.

## Correctness invariants

- Use `SetThreadExecutionState` on one long-lived owning thread for activation,
  updates, and cleanup. Event callbacks must not accidentally move calls to
  arbitrary workers. Make thread ownership explicit in the design.
- Always include `ES_CONTINUOUS`. Add `ES_SYSTEM_REQUIRED` when active and
  `ES_DISPLAY_REQUIRED` only when both active and display preference is on.
  Inactive means `ES_CONTINUOUS` alone, regardless of display preference.
- Check every power API return value. A zero return is failure. Commit the
  effective application state and success UI only after the OS call succeeds;
  failed activation or deactivation must not falsely show success. Report
  actionable errors even when the release executable has no console.
- Explicitly clear requests on normal exit from the owning thread. Use RAII
  for owned resources where appropriate, but do not hide cleanup failures or
  depend on destructors running after abrupt termination.
- Keep unsafe FFI small, documented with safety reasoning, and wrapped in a
  safe interface. Keep power management independent of tray code.
- Enforce one instance per interactive user session using a lightweight
  session-scoped mechanism. A duplicate launch exits without another icon.
- Store settings per user. Read actual startup registration for its menu
  state; report failed writes and handle missing/malformed settings sensibly.
- Startup registration must be opt-in, per user, correctly quote the executable
  path, and point to a stable location. Do not silently register a disposable
  Cargo output path. Do not modify machine-wide startup settings.

## Code and validation

Start with the handover's suggested `main.rs`, `power.rs`, `tray.rs`, and
`settings.rs` boundaries as each becomes necessary. Avoid empty scaffolding,
generic frameworks, extra crates, and abstractions with no current consumer.
Commit `Cargo.lock` for the executable once Cargo is initialized.

For Rust changes, once the toolchain and manifest exist, run applicable checks:

```powershell
cargo fmt --check
cargo check --locked
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

Create or deliberately update the lockfile before using `--locked`; inspect
dependency changes. Use focused tests for state transitions, flags, and failure
behavior when useful. Do not build a tray automation framework or tests that
merely duplicate implementation. Documentation-only changes need link and
consistency checks, not fabricated build results.

Verify Windows behavior externally using `powercfg /requests` and the manual
matrix in the workflow. Compilation, mocks, and screenshots alone do not prove
that Windows is honoring or releasing requests. Record environment limits and
exact remaining manual steps. Before a release, also run
`cargo build --release --locked` and test the standalone executable.

Release readiness requires review of thread ownership, error paths, cleanup,
tray behavior, startup registration, assets, and the documented test evidence.
Do not claim Windows 10/11 coverage from testing only one OS version.
