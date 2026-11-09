# RedEye development workflow

## Outcome and source of truth

Build the v0.1 utility described in [README.md](../README.md), incrementally,
while teaching the Rust and Windows concepts behind each meaningful change.
[AGENTS.md](../AGENTS.md) supplies the standing instructions. This document
defines responsibility, delivery gates, and evidence. [Icon direction](ICONS.md)
defines the original visual identity. Keep these documents small and current.

## Roles

| Role | Owns | Required output |
|---|---|---|
| Lead / integrator | Scope, milestone plan, architecture choices, integration, user communication | A bounded task with acceptance criteria; coherent integrated changes; decision and verification summary |
| Rust / Windows engineer | Power API, owning thread, event loop, tray integration, settings, startup, single instance | Readable implementation, small safe FFI boundary, meaningful focused tests, concise learning notes |
| UX / asset designer | Tray interactions, menu wording, error presentation, original eye icons and DPI variants | Active/inactive assets and an in-context check at actual tray sizes; behavior consistent with the product scope and icon direction |
| Correctness reviewer | Diff review, unsafe reasoning, ownership, failure paths, dependency cost, scope | Findings with file locations, impact, and required fixes; explicit assessment of milestone acceptance criteria |
| Validation / release engineer | External Windows behavior, regression checks, portable build, README and removal instructions | Commands and observed results, manual-test evidence, remaining limitations, reproducible release procedure |

Learning and documentation are shared responsibilities. The implementation
engineer explains the design; the reviewer checks that the explanation matches
the code; the lead keeps the user informed.

The main agent leads; implementation, independent review, and validation use
separate specialist subagents for almost every code change. This is a standing
project requirement, so the user need not explicitly request agents each time.
Run dependent roles sequentially. Parallel work is appropriate only for
independent tasks, such as asset design and a bounded power module, with
separate file ownership. A review agent reviews the integrated diff and does
not rewrite it unseen. Self-review does not satisfy independent review.

## When the subagent workflow is required

Use the full implementation -> independent review -> validation sequence for
application code, tests, executable scripts, build/CI configuration, dependency
and lockfile changes, and behavior-affecting settings or assets. Include small
fixes, refactors, and one-line behavior changes. Use `windows_engineer` for
implementation, `correctness_reviewer` for independent review, and
`release_validator` for checks of the final result. Add `ux_designer` for visual
design or icon work. The lead owns scope, integration, and the final report.

The lead may work directly on read-only investigation or changes limited to
prose documentation, agent/workflow instructions, comments, or formatting that
cannot affect executable behavior. Documentation tests, tool directives,
functional configuration, and changes mixed with code are not exempt. State
the exception used in the task summary. If unsure, use the full workflow.

Delegate automatically within the authorized task scope under `AGENTS.md`;
do not ask for repeated delegation permission. These standing instructions
supply orchestration without an additional workflow skill. If a required role
is unavailable, report it and continue unaffected investigation rather than
silently replacing independent review with self-review.

## Codex agent definitions

The lead/integrator is the main conversation agent. The four reusable specialist
roles have project-scoped definitions:

| Agent name | Definition |
|---|---|
| `windows_engineer` | [windows_engineer.toml](../.codex/agents/windows_engineer.toml) |
| `ux_designer` | [ux_designer.toml](../.codex/agents/ux_designer.toml) |
| `correctness_reviewer` | [correctness_reviewer.toml](../.codex/agents/correctness_reviewer.toml) |
| `release_validator` | [release_validator.toml](../.codex/agents/release_validator.toml) |

`AGENTS.md` defines shared repository guidance; these TOML files define named
specialists Codex can spawn. Each includes the documented `name`, `description`,
and `developer_instructions` fields. They omit model and reasoning overrides
to inherit the parent/default settings, and do not override permissions.
The reviewer's instruction to avoid edits is a role boundary, not a separate
read-only sandbox.

Current [official subagent documentation](https://learn.chatgpt.com/docs/agent-configuration/subagents)
supports standalone files under `.codex/agents/` and enables subagents by default.
A `.codex/config.toml` is optional for defaults or concurrency settings; this
project currently needs no overrides. Do not add legacy registration stanzas
or model identifiers without verifying the installed client's requirements.

Example requests, once the relevant milestone is authorized (naming agents is
optional because the standing workflow already requires them for code changes):

- "Delegate milestone 1 power control to windows_engineer, then have
  correctness_reviewer review the integrated change."
- "Have ux_designer create the original active/inactive tray icons while
  windows_engineer works on the separate power module."
- "Have release_validator check this milestone and report any Windows tests
  that still require manual verification."

Use a current client that supports this documented schema. If new definitions
are not discovered in an existing conversation, start a fresh Codex session in
the repository and verify discovery before relying on the names. Creating files
alone is not evidence that this running session has loaded or exercised them.

## Work loop

1. **Frame:** Read the relevant product requirements in `README.md` and current
   code. State the milestone, acceptance criteria, affected files, and technical
   unknowns.
2. **Verify assumptions:** Inspect tools; check current official API/library
   documentation when selecting dependencies. Resolve the smallest uncertain
   integration with a focused experiment before committing the architecture.
3. **Implement:** Delegate a coherent change to `windows_engineer`, with
   `ux_designer` for visual work as needed. Require explicit resource ownership,
   error behavior, and an explanation of the Rust concepts introduced.
4. **Review:** After integration, have a separate `correctness_reviewer`
   inspect the actual final diff against acceptance criteria and `AGENTS.md`.
   Send fixes to the implementation owner and have the reviewer recheck them.
   Review subsequent functional changes as well; do not rely on a stale review.
5. **Validate:** Have `release_validator` run applicable Rust and external
   Windows checks on the resulting change. Record pass, fail, or not run, with
   a reason. Reuse evidence for unchanged artifacts and recheck affected
   behavior after fixes. Native milestone gates still require native evidence.
6. **Hand off:** Summarize changed behavior, evidence, limitations, and the next
   step. Update milestone status and material decisions here.

A delegated task brief must include its role, goal, owned files, constraints,
acceptance criteria, dependencies on other work, and expected evidence. Its
return must identify changes, checks, unresolved issues, and teaching notes.
The lead verifies integration and the evidence returned for the combined
changes; it need not rerun passing checks on an unchanged artifact. Any
functional integration fixes go through the same implementation, review, and
validation sequence. Do not mark unresolved correctness findings complete.

Tell implementing agents that they are sharing the repository and must not
revert others' work. Their task brief does not authorize them to delegate
recursively unless the lead explicitly assigns that responsibility.

Use the existing session or task description for these briefs. Do not create
a ticket hierarchy, mandatory approval ceremony, or a document for every edit.

## Repository changes and CI

Use a feature branch and a PR to `main`, including for changes made by agents.
`@tbrownhe` owns every path through [CODEOWNERS](../.github/CODEOWNERS), including
that file, CI, and rule definitions. External contributions require that owner's
approval; new commits dismiss stale approvals. Review conversations must be
resolved before merging.

Two rulesets target only `refs/heads/main`:

- [main-integrity](../.github/rulesets/main-integrity.json) requires a PR,
  resolved conversations, and an up-to-date successful `Windows CI` check from
  the GitHub Actions app. It blocks deletion/force pushes and has no bypass.
- [owner-review](../.github/rulesets/owner-review.json) requires one approval and
  code-owner review. Only `@tbrownhe` has a PR-only review bypass, as explicitly
  chosen by the user because GitHub does not permit self-approval. It cannot
  bypass the independent integrity/CI ruleset or permit direct pushes.

These JSON files record the intended GitHub settings for review. Committing
them does not apply them automatically; apply changes through repository
administration and verify the effective server-side rules afterward.

The [CI workflow](../.github/workflows/ci.yml) uses the stable check name
`Windows CI`. Coordinate any rename with the required-check rule. It runs the
required Rust checks and release build on `windows-2025-vs2026` with Rust
`1.99.1-x86_64-pc-windows-msvc`, read-only contents permissions, and no persisted
checkout credentials. First-party actions are pinned to verified commit SHAs.
PR runs cancel superseded runs; each main/manual run gets an independent group.
Successful runs upload only the executable and checksum for seven days.
No application GUI or power behavior is claimed from CI's unit/build checks.

## Milestones and exit gates

| Stage | Work | Evidence required to complete |
|---|---|---|
| 0. Project workflow and environment | Establish roles and guardrails; inspect Rust/MSVC availability; verify selected APIs before implementation | Workflow in place; tools and unresolved setup needs recorded; dependency decisions supported before use |
| 1. Native power control | Minimal console program; safe wrapper; one owning thread; explicit clear and errors | Build/checks pass; `powercfg /requests` shows the request while enabled and its removal after clear/exit; failure handling reviewed |
| 2. Native tray | Tray icon, native menu, event loop, Exit; retain working power lifecycle | Launch/menu/exit work on Windows; no request left after exit; chosen integration demonstrates required left/right event distinction before expanding it |
| 3. Interactive state | Toggle, display preference, icon/tooltip/checkmark updates; original icon assets | Four-state matrix below verified; repeated toggles remain consistent; API failure cannot falsely update success UI; icon states readable at tray size |
| 4. Persistence and lifecycle | Per-user settings, opt-in sign-in registration, session single instance | Preferences survive restart; actual registration agrees with menu; quoting/stable-path behavior verified; duplicate launch exits; cleanup paths checked |
| 5. Distribution | Standalone release exe, embedded assets, README, build/usage/removal instructions | Rust checks and release build pass; copied exe runs without Cargo or loose assets; no console/elevation; applicable manual matrix completed with OS coverage recorded |

Advance in this order. A stage with an unperformed native check is implemented
but unverified, not fully validated. Document an exact follow-up check if the
environment cannot perform it; do useful independent work without inventing
results. Pause expansion when an unresolved correctness issue would undermine
later work.

Stage 0 environment setup is not complete merely because the documents exist.
Verify a real Windows MSVC build before treating the build environment as ready.
Do not add future-stage modules until they serve the current milestone.

## State contract

`keep_display_on` is a saved preference. `active` is the effective session
state after a successful OS call. Keep these concepts separate.

| Active | Display preference | Requested flags | Expected requests |
|---|---|---|---|
| No | Off | `ES_CONTINUOUS` | Neither system nor display |
| No | On | `ES_CONTINUOUS` | Neither; retain checked preference |
| Yes | Off | `ES_CONTINUOUS \| ES_SYSTEM_REQUIRED` | System only |
| Yes | On | `ES_CONTINUOUS \| ES_SYSTEM_REQUIRED \| ES_DISPLAY_REQUIRED` | System and display |

Review both activation and deactivation failure. Preserve the last successfully
applied state and present an actionable error; do not persist or render a
successful OS transition that did not occur. Review failed tray updates and
settings/startup writes as well, so the user can understand any partial failure.

## Windows validation matrix

Capture `powercfg /requests` before launching, after each relevant transition,
and after exit. Identify RedEye's contribution separately from other programs'
requests; another application's request does not imply RedEye failed to clear.
If the diagnostic command requires elevation on the test machine, record that
as a diagnostic requirement, not an application privilege requirement.

| Check | Expected evidence |
|---|---|
| Fresh launch | One icon; active tooltip/menu; system request; display allowed off by default |
| Display mode on/off while active | Display request appears/disappears; system request remains |
| Disable with either display preference | Both requests clear; inactive icon and tooltip; preference remains accurate |
| Change display preference while inactive, then enable | No request while inactive; enabling applies the chosen preference |
| Repeated left-click and menu toggles | Exactly one intended transition per action; OS requests, icon, tooltip, checkmarks agree |
| Right-click / dismiss menu | Native context menu; opening/dismissing does not toggle awake state |
| Power API failure | Focused failure test or documented controlled experiment; no false success UI; actionable error |
| Exit / forced termination | Requests attributable to RedEye disappear; normal exit removes tray icon; note any shell icon cleanup delay after forced termination |
| Second launch | One process/icon per interactive user session; other sessions are not unnecessarily blocked |
| Settings restart / malformed config | Display preference restored on restart; active starts on; invalid data handled without a crash |
| Startup enable/disable and real sign-in | Correct per-user registration, quoted stable path including spaces, actual sign-in launch, menu reflects registration; default is opt-out |
| Windows lock / explicit sleep | Lock is unaffected; explicit sleep is not prevented; resume behavior checked |
| Tray readability and DPI | Both icon states recognizable at 16x16 and scaled tray sizes; tooltip/menu provide textual state |
| Explorer restart | Check whether the icon returns and remains usable; document or fix any recovery limitation before release |
| Portable release | Copied exe runs as a standard user without Cargo, console, or external assets; exit and removal instructions work |
| Idle resource use | Observe CPU/memory after settling; no repeating keep-awake work or busy loop; record measurements and context |

Record date, Windows version/build, architecture, Rust version, artifact or
commit, action, expected result, actual result, and status for milestone evidence.
Use a short verification section in the task/PR; retain release evidence under
`docs/` when there is an artifact to identify. State separately which Windows
10 and Windows 11 versions were actually tested.

## Initial checkpoint — 2026-11-08

- Reviewed the initial product requirements and visual direction.
- Added the workflow and repository instructions. No application code or Cargo
  manifest has been created; dependency versions and APIs remain to be verified.
- Added four project custom-agent TOML definitions under `.codex/agents/` after
  checking the official schema. The local CLI reports
  `codex-cli 0.164.0-alpha.11.1`. Definitions have not been exercised by spawning
  the named agents in a fresh session.
- In the current agent PowerShell session, `rustc`, `cargo`, `rustup`, `cl`, and
  `link` were not found on PATH. This does not establish that they are absent
  from the machine. Next, locate installed tools and verify an MSVC build;
  address installation only if needed and within the user's authorized scope.
- Git reports an unborn `main` branch. Its ownership differs from the agent
  account, so read-only inspection used a command-scoped `safe.directory`
  exception for this exact repository; no global Git configuration was changed.
- Stage 0: workflow complete; build environment and API verification pending.
  Stages 1–5: not started. No build or Windows behavior tests have run.

## Toolchain verification — 2026-11-08

This supersedes the initial checkpoint's pending build-environment status.

| Component | Verified installation |
|---|---|
| Rust / Cargo | `rustc 1.99.1`, `cargo 1.99.1`; default `stable-x86_64-pc-windows-msvc` |
| Rust checks | Clippy `0.1.99` and rustfmt `1.10.0-stable` installed and exercised |
| C++ build tools | Visual Studio Community 2026, MSVC toolset `14.51.36231`, x64 compiler/linker |
| Windows SDK | `10.0.26100.0`; x64 system/UCRT libraries and Windows headers present |

Created a disposable Cargo project outside this repository with no external
dependencies. A direct `kernel32!GetCurrentProcessId` import matched Rust's
process ID in both its test and release executable. The following passed:

```powershell
cargo generate-lockfile --offline
cargo fmt --check
cargo check --locked --offline
cargo clippy --locked --offline --all-targets -- -D warnings
cargo test --locked --offline
cargo build --release --locked --offline
```

The one native API test passed, and the release executable ran successfully.
Rust discovered MSVC and linked successfully from ordinary PowerShell without
loading a Visual Studio developer environment.

Also compiled and ran a C++ program including `windows.h` and calling
`GetCurrentProcessId`, using `cl.exe /nologo /W4 /WX /EHsc` after loading the
installed Visual Studio DevShell module and entering its x64 environment.
The `Launch-VsDevShell.ps1` helper was blocked by this shell's script execution
policy; importing the installed binary module worked without changing policy.

Before restarting VS Code, the agent's inherited PATH lacked Cargo and the
probe commands needed this process-local adjustment:

```powershell
$env:PATH = (Join-Path $env:USERPROFILE '.cargo\bin') + ';' + $env:PATH
```

After the user restarted VS Code, the agent resolved `rustup`, `rustc`, and
`cargo` from `C:\Users\tbrow\.cargo\bin` without any PATH adjustment.
`rustc --version --verbose`, `cargo --version`, `rustup show active-toolchain`,
`cargo clippy --version`, and `cargo fmt --version` all succeeded. The user
also confirmed Rust version commands work in the integrated terminal.

No permanent PATH or execution-policy changes were made by the agent.
Rustup emitted a home-path canonicalization warning in the agent environment;
it did not prevent any build, lint, test, or executable from succeeding.

Build environment: verified for native x64 Windows development. Stage 0 still
needs library/API selection verification before application implementation.
Stages 1–5 remain unstarted; these probes did not test RedEye power/tray behavior.

## Decision record

| Date | Decision | Reason / evidence |
|---|---|---|
| 2026-11-08 | Use root `AGENTS.md` plus this workflow; five roles can run sequentially | Clear instructions and accountable review without a large process for a small learning project |
| 2026-11-08 | Add four named specialists under `.codex/agents/`; main agent remains lead; inherit model and permission settings | User expected reusable subagents; [official docs](https://learn.chatgpt.com/docs/agent-configuration/subagents) specify standalone TOML definitions; no project config overrides needed |
| 2026-11-08 | Supersede the initial sequential single-agent default: require delegated implementation, independent review, and validation for almost every code change | User requested the subagent workflow for almost any code change; narrow exceptions cover read-only work and nonfunctional documentation/instruction/comment/formatting changes; no extra workflow skill required |
| 2026-11-08 | Defer crate/event-loop/settings-format choices until environment and current APIs are verified | Product requirements make Windows click handling, thread ownership, small dependencies, and compatibility selection criteria |
| 2026-11-08 | Milestone 1 uses a direct, scalar `extern "system"` Kernel32 import and no crates | Microsoft API documentation and local SDK 10.0.26100.0 `WinBase.h`/`WinNT.h` confirm the DWORD signature, flags, and Windows compatibility; one API does not need a binding dependency. Reevaluate bindings before tray implementation. |
| 2026-11-08 | Main owns the power request; the wrapper is neither `Send` nor `Sync`; cleanup is explicit | Thread-affine requests must not migrate. `PhantomData<Rc<()>>` prevents safe cross-thread movement/sharing without allocation. Explicit cleanup can report failures that a destructor could not return. |
| 2026-11-08 | For milestone 2 use `windows-sys` 0.61.2 with a native hidden window and blocking message loop | [Official manifest](https://docs.rs/crate/windows-sys/0.61.2/source/Cargo.toml.orig): MSRV 1.71, no default features, only `windows-link` underneath. Enable `Win32_Foundation`, `Win32_Graphics_Gdi`, `Win32_System_LibraryLoader`, `Win32_UI_Shell`, and `Win32_UI_WindowsAndMessaging`. [Rust target support](https://doc.rust-lang.org/rustc/platform-support.html) covers x64 MSVC Windows 10+. |
| 2026-11-08 | Evaluate `windows` 0.62.2 and `tray-icon` 0.26.1; prefer direct native tray bindings for this Windows-only utility | [windows manifest](https://docs.rs/crate/windows/0.62.2/source/Cargo.toml.orig) requires Rust 1.82 and more support crates. [tray-icon manifest](https://github.com/tauri-apps/tray-icon/blob/tray-icon-v0.26.1/Cargo.toml) requires Rust 1.90 and adds menu/channel/error helpers while still needing an event loop. Its [Windows source](https://github.com/tauri-apps/tray-icon/blob/tray-icon-v0.26.1/src/platform_impl/windows/mod.rs) supports distinct left/right events, so it is viable; direct bindings keep error checks and owning-thread dispatch explicit. |
| 2026-11-08 | Use a hidden top-level window and `GetMessageW`, with no app-state pointer in the window procedure | [GetMessageW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmessagew) blocks without polling and has distinct error/quit/message results. Dispatch must avoid mutable aliasing during reentrant native callbacks. [Shell_NotifyIconW](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shell_notifyiconw) supports native icon/menu events on Windows 10/11. |
| 2026-11-08 | Original geometric eye icons use PNG-compressed ICO frames at 16/20/24/32/48/64/128/256 pixels | `ux_designer` inspected the reference and produced deterministic drawing source and actual-size previews. Active/inactive differ in brightness and lens shape as well as color. See [icon notes](ICONS.md); native tray/DPI verification is still required. |
| 2026-11-08 | Main coordinates typed tray events, effective power state, and display preference in milestone 3 | A blocking `next_event` ends the tray borrow before the OS transition. Failed power calls preserve state/visuals and report a recoverable native error. Partial visual update failures exit through explicit cleanup to avoid leaving misleading UI. Checkmarks use checked [SetMenuItemInfoW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setmenuiteminfow) calls. |
| 2026-11-08 | Milestone 4 uses native HKCU registry storage, without another crate | `Software\RedEye\KeepDisplayOn` is a strict DWORD preference. Startup uses only the named `RedEye` value under the per-user Run key. [Registry reads](https://learn.microsoft.com/en-us/windows/win32/api/winreg/nf-winreg-reggetvaluew) return status codes directly. Missing data defaults off; malformed data or read/write failure must be reported accurately. Add only Registry, Threading, and Security/Authorization features to the existing bindings. |
| 2026-11-08 | Startup points to an explicitly quoted stable executable; known Cargo/temp paths are refused | [Run-key documentation](https://learn.microsoft.com/en-us/windows/win32/setupapi/run-and-runonce-registry-keys) limits the command to 260 characters. Preserve Windows Unicode paths. Installation remains a manual portable copy, with startup disabled by default. Existing actual registration is read afresh when opening the menu, including registrations to another copy. |
| 2026-11-08 | Single-instance guard is a session-local mutex named with the current user SID | [CreateMutexW](https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-createmutexw) with no initial ownership and immediate `ERROR_ALREADY_EXISTS` handling needs no waits or IPC. [Local namespaces](https://learn.microsoft.com/en-us/windows/win32/termserv/kernel-object-namespaces) separate sessions; the SID also separates users. Owned token/mutex handles stay alive for the required scope. |
| 2026-11-08 | Restore the current tray icon after Explorer's `TaskbarCreated` broadcast | The hidden top-level window receives the [documented notification](https://learn.microsoft.com/en-us/windows/win32/shell/taskbar#taskbar-creation-notification). Re-add the current icon and callback version on the owning thread; do not refresh power requests. A failed restore must report and cleanly exit. |
| 2026-11-08 | Plan static CRT linkage and the GUI subsystem for the portable release | Read-only `dumpbin` inspection found `VCRUNTIME140.dll` in the debug imports. Rust supports [static CRT linkage](https://doc.rust-lang.org/reference/linkage.html#static-and-dynamic-c-runtimes) on x64 MSVC, configured through [target rustflags](https://doc.rust-lang.org/cargo/reference/config.html#targettriplerustflags). The [windows_subsystem attribute](https://doc.rust-lang.org/reference/runtime.html#the-windows_subsystem-attribute) removes the release console. Final imports/subsystem and standalone behavior must be verified after milestone 4 completes. |
| 2026-11-08 | Initialize system-DPI awareness before any window, warning, or icon metric | Default DPI-unaware metrics could select a 16-pixel source even on a scaled desktop. [SetProcessDPIAware](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setprocessdpiaware) is supported since Vista and avoids requiring newer Windows 10 APIs; tolerate existing awareness and check initialization success. Microsoft [prefers a manifest](https://learn.microsoft.com/en-us/windows/win32/hidpi/setting-the-default-dpi-awareness-for-a-process), but this small single-threaded app can initialize before all UI without adding a build script. System-DPI-at-launch is the supported approach; multi-monitor scaling remains a documented validation limit. |
| 2026-11-08 | Embed the existing active eye as a PE application-icon resource, as requested by the user | [winresource 0.1.34](https://docs.rs/crate/winresource/0.1.34) with default features disabled adds only `version_check` 0.9.5 at build time, with no new runtime dependencies. Its [official source](https://github.com/BenjaminRi/winresource) verifies `set_icon`/`compile` and Windows SDK discovery. The crate declares no MSRV; the project's actual Rust 1.99.1 was tested. A small `build.rs` fails on resource-compiler errors and tracks source ICO/tool overrides. SDK `rc.exe` is a build requirement, not a runtime dependency. All eight [icon resource](https://learn.microsoft.com/en-us/windows/win32/menurc/icon-resource) sizes are retained. |
| 2026-11-08 | Add PR-based development, owner review, and mandatory Windows CI as independent main-branch rulesets | [GitHub rulesets](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets) support layered requirements. The user chose a PR-only review bypass for `@tbrownhe`; PR/CI, conversation resolution, and force-push/deletion protection have no bypass. [CODEOWNERS](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-code-owners) assigns every path to the owner. Source JSON is applied through the [REST rules API](https://docs.github.com/en/rest/repos/rules?apiVersion=2026-03-10), not by CI. |
| 2026-11-08 | Run checks and release builds on `windows-2025-vs2026` with Rust 1.99.1, using SHA-pinned first-party actions | The [official image inventory](https://github.com/actions/runner-images/blob/main/images/windows/Windows2025-VS2026-Readme.md) supplies x64 MSVC/SDK and Rust tools. Official tag refs and action manifests verified checkout v7.0.1 (`3d3c42e5aac5ba805825da76410c181273ba90b1`) and upload-artifact v7.0.2 (`cf430e030ddbb5b0abf93d22962f4752f3646cd9`). No extra app crates, cache action, third-party toolchain action, publishing, or privileged PR event are needed. |

Add a short row for a material decision, including source links when verifying
external APIs. Revisit a choice when evidence changes; avoid speculative design.

## Implementation checkpoint — 2026-11-08

- Lead inspected repository instructions, product requirements, tracked files,
  and Git status before implementation, preserving the initial project inputs.
  Git inspection uses
  `-c safe.directory=C:/Users/tbrow/dev/RedEye` for this repository only.
- Rust and Cargo still resolve from `C:\Users\tbrow\.cargo\bin` without PATH
  changes: Rust/Cargo 1.99.1, `stable-x86_64-pc-windows-msvc`. No tools were
  reinstalled. The native environment reports Windows 11 Home, version
  10.0.26300, build 26300, x64; Windows 10 has not been tested.
- Milestone 1 implementation is assigned to `windows_engineer`; independent
  `correctness_reviewer` review and `release_validator` checks follow it.
  Later milestones remain gated on the native power-request evidence.
- The lead checked Microsoft's current
  [SetThreadExecutionState reference](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-setthreadexecutionstate):
  requests are thread-owned, continuous requests require no periodic refresh,
  zero indicates failure, and clearing uses `ES_CONTINUOUS` alone. The API is
  available on Windows 10/11 through Kernel32. It does not document a
  `GetLastError` contract, so failure messages must not invent an OS error code.
- `powercfg /requests` was attempted inside and outside the sandbox. The
  outside-sandbox diagnostic reports that an elevated command prompt is
  required. This is an unavailable diagnostic privilege, not a requirement
  to elevate RedEye. Native request appearance/removal is **not yet verified**.
  User assistance with the elevated diagnostic has been requested.
- Milestone 1 now has a console implementation in `src/main.rs` and
  `src/power.rs`, with no third-party dependencies. It starts active, accepts
  `on`, `off`, `status`, and `quit`, and exits on EOF. `quit`, EOF, and returned
  I/O/API errors all reach explicit cleanup. Ctrl+C/window closure are abrupt
  termination paths; use `quit` when testing explicit cleanup.
- Implementation checks passed: `cargo fmt --check`, `cargo check --locked`,
  `cargo clippy --locked --all-targets -- -D warnings`, `cargo test --locked`
  (two focused tests), and `cargo build --locked`. Console on/off/on/quit and
  EOF smoke runs returned zero and reported explicit clear. These results
  prove successful API returns, not external Windows request observation.
- Independent `correctness_reviewer` review found no correctness defects.
  `release_validator` repeated the Rust checks and console smoke tests on the
  unchanged executable; all passed. The user supplied elevated diagnostics
  showing an empty baseline, RedEye's SYSTEM request on launch, no request
  after `off`, its return after `on`, persistence after 30 seconds, and removal
  after `quit`; DISPLAY stayed clear. **Milestone 1 is complete on Windows 11.** See
  [validation evidence](VALIDATION.md) for the artifact hash, checks, limits,
  and remaining platform limits. Milestone 2 is now in implementation;
  milestones 3–5 have not started.
- The lead directly maintained prose documentation under the documented
  nonfunctional exception; application code, tests, manifest, lockfile, and
  ignore configuration followed delegated implementation, separate review,
  and specialist validation.

## Milestone 2 checkpoint — 2026-11-08

- Native tray integration, original embedded active icon, version-4 shell
  callbacks, native menu, and Exit are implemented. The hidden top-level
  window and blocking message loop share the main thread with power ownership.
  Callback actions are queued without borrowing application state; queue
  borrows end before potentially reentrant native calls.
- Independent review found no correctness defects. Specialist validation
  passed formatting, locked check/build, Clippy, and all three tests. Original
  icon generation is reproducible; frame decoding and bounds checks passed.
- User confirmed the tooltip, native menu, dismissal, and Exit, and supplied
  console output showing three alternating left/right interactions without
  changing power state. Elevated diagnostics showed only RedEye's SYSTEM
  request while active and no requests after Exit, which reported explicit
  clear. **Milestone 2 is complete on Windows 11.** See
  [validation evidence](VALIDATION.md) for the exact artifact and limitations.
- Milestone 3 now adds interactive awake/display state. Main will receive
  commands from a blocking tray event method, apply power changes first, and
  render only the successfully applied state. No new dependencies are needed.
  Preferences, startup registration, and single-instance handling remain
  milestone 4 work; distribution remains milestone 5.

## Milestone 3 checkpoint — 2026-11-08

- Awake and display controls, both embedded icon states, exact tooltips, and
  native checkmarks are implemented. Main applies the OS transition before
  committing the preference and rendering. Failed transitions preserve prior
  state and report a recoverable error; failed rendering exits with cleanup.
- Independent review found no defects. Specialist validation passed all Rust
  checks, four focused tests, and the debug build. Dependencies/assets did not
  change. Failure evidence uses injected returns, not a real OS API failure.
- User confirmed the native interaction/UI procedure and supplied eight
  ordered `powercfg /requests` observations: SYSTEM only; SYSTEM + DISPLAY;
  neither after disabling; neither while changing display off and on inactive;
  both after enabling; SYSTEM only after clearing display; neither after Exit.
  Console transitions agree, including retained inactive preference and
  explicit cleanup. **Milestone 3 is complete on Windows 11.**
- Milestone 4 now adds per-user preference persistence, opt-in startup,
  session/user single-instance protection, and Explorer icon recovery. The
  final two stages remain unverified until their own checks are recorded.

## Milestone 4 checkpoint — 2026-11-08

- Added `settings.rs` for HKCU preferences/startup and `instance.rs` for the
  SID-qualified local mutex. Only binding features changed; crate versions and
  the lockfile package graph are unchanged. Explorer recovery re-adds the
  current icon/version on the main thread without another power call.
- Independent review found no defects. Specialist validation passed all Rust
  checks, seven tests, the debug build, and an identical copied test executable
  with a space in its path. User confirmed all six native procedure items:
  duplicate launch, Cargo startup refusal, preference restart with external
  SYSTEM + DISPLAY requests, malformed setting warning/fallback and repair,
  quoted stable-path startup enable/readback/disable, and cleanup after Exit.
- Duplicate behavior is intentional: the newly launched process exits; the
  existing instance continues holding its selected request. A duplicate is
  not an Exit command. **Milestone 4's core exit gate is complete on Windows 11.**
- Real sign-in, actual Explorer restart, cross-session isolation, high-DPI
  changes, Windows 10, and lock/explicit-sleep behavior remain unperformed
  release-readiness checks. Their exact manual steps remain in
  [validation evidence](VALIDATION.md); no automated result substitutes for
  them. Milestone 5 now prepares a portable release candidate with these
  limitations clearly retained.

## Milestone 5 checkpoint — 2026-11-08

- Release builds use the GUI subsystem and static CRT. Checked process DPI
  initialization precedes every window/dialog/metric query. No new crates,
  build scripts, profile tuning, service, or elevation requirement were added.
- Independent final/cumulative source review found no correctness defects.
  Specialist validation passed formatting, locked check, Clippy, seven tests,
  and `cargo build --release --locked`. PE inspection confirms x64 GUI
  subsystem 2 and only Windows system DLL imports, with no separate VC runtime
  dependency. This pre-resource executable was 407,040 bytes.
- A copied executable, self-contained usage/removal/build instructions,
  checksums, and ZIP are prepared under `dist/`. The binary embeds both icon
  assets and needs no loose assets or Cargo. Its exact SHA-256 and package
  evidence are recorded in [validation evidence](VALIDATION.md).
- User confirmed the copied executable's no-console/no-elevation launch,
  tray/state behavior, and Exit/request cleanup. This remains a release
  candidate with the previously listed manual release-readiness checks
  unperformed. No release has been published or tagged.
- The user then requested the red eye on the executable itself. PE icon
  resources are a separate bounded follow-up: reuse the original active ICO,
  verify the resource build tooling, independently review, rebuild, and
  validate the resulting resources/hash before final packaging.

## Executable-icon follow-up — 2026-11-08

- Reused the original active ICO as the default PE application icon, retaining
  all eight sizes for Explorer and shortcuts. Runtime code and tray visuals
  did not change. `build.rs` uses the installed SDK resource compiler through
  the verified minimal build helper; compile errors fail the build.
- Separate review found no defects. Required Rust checks, seven tests, and
  the release build passed. Independent PE parsing found the group icon and
  eight byte-identical source frames, no manifest override, and the same x64
  GUI subsystem/system-only imports. The updated executable is 474,624 bytes;
  [validation evidence](VALIDATION.md) identifies its hash and final package.
- Prior user-confirmed portable runtime evidence is reused for unchanged
  application behavior. Windows native APIs successfully extracted the eye
  at 16/32/64 pixels without executing the application. The lead inspected
  this preview. Final package checks and all ZIP payload hashes passed.
- The local portable v0.1 candidate is complete and ready to copy to a stable
  folder. Explorer's visible cached icon and the broader remaining manual
  release checks remain unverified, with exact follow-up steps recorded.
  Runtime code is unchanged by the resource-only follow-up; no new application
  launch is claimed for it. No release was published or tagged.

## Documentation consolidation — 2026-11-08

- Product scope, exclusions, deliverables, and module responsibilities now
  live in `README.md`; visual direction and optional brand text live in
  `docs/ICONS.md`. This workflow retains milestone gates, decisions, and
  validation responsibilities, with evidence in `docs/VALIDATION.md`.
- Retired the obsolete initial planning inputs at the user's request. Repository
  and specialist instructions now read the maintained product/visual docs.
- The lead used the nonfunctional documentation/agent-instruction exception.
  Application source, build configuration, generated icons, and release
  artifacts did not change. All 29 local documentation links resolve; all four
  specialist TOML files parse with only intended instruction-text changes.
  Reference scans are clear and release artifact hashes are unchanged. Rust
  builds/native tests were not repeated for this documentation-only change.

## PR and CI setup — 2026-11-08

- Applied and read back both active GitHub rulesets: main-integrity
  [26833257](https://github.com/tbrownhe/RedEye/rules/26833257) and owner-review
  [26833258](https://github.com/tbrownhe/RedEye/rules/26833258). The five effective
  rules match the reviewed JSON, including no CI/integrity bypass and only the
  explicitly chosen owner PR review exception. Existing read-only workflow
  permissions and first-time contributor workflow approval were preserved.
- Used `ci/protect-main` and [PR #1](https://github.com/tbrownhe/RedEye/pull/1)
  to install the workflow and global CODEOWNERS mapping. The owner exception
  supports this owner-authored bootstrap; it does not permit direct main pushes
  or merging a head without successful CI.
- Separate review found no defects. Local Rust checks, seven tests, release
  build, exact policy/checksum checks, and verified actionlint 1.7.12 passed.
  The [first hosted run](https://github.com/tbrownhe/RedEye/actions/runs/42572845456)
  independently passed all checks on the documented Windows/MSVC runner. The
  validator checked the downloaded executable/checksum, GUI/x64 metadata,
  system-only imports, and original icon resources without running the app.
- [Validation evidence](VALIDATION.md) identifies the exact tested head,
  artifact hashes, and unchanged native-test limitations. Later documentation
  commits must pass their own CI before merge. [GitHub CI history](https://github.com/tbrownhe/RedEye/actions/workflows/ci.yml)
  records current per-commit PR/main results; build success does not substitute
  for the native Windows manual matrix.
