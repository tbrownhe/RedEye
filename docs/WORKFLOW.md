# RedEye development workflow

## Outcome and source of truth

Build the v0.1 utility described in [the handover](../handover.md), incrementally,
while teaching the Rust and Windows concepts behind each meaningful change.
[AGENTS.md](../AGENTS.md) supplies the standing instructions. This document
defines responsibility, delivery gates, and evidence. Keep it small and current.

## Roles

| Role | Owns | Required output |
|---|---|---|
| Lead / integrator | Scope, milestone plan, architecture choices, integration, user communication | A bounded task with acceptance criteria; coherent integrated changes; decision and verification summary |
| Rust / Windows engineer | Power API, owning thread, event loop, tray integration, settings, startup, single instance | Readable implementation, small safe FFI boundary, meaningful focused tests, concise learning notes |
| UX / asset designer | Tray interactions, menu wording, error presentation, original eye icons and DPI variants | Active/inactive assets and an in-context check at actual tray sizes; behavior consistent with the handover |
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

1. **Frame:** Read the relevant handover requirements and current code. State
   the milestone, acceptance criteria, affected files, and technical unknowns.
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

- Read the complete handover. Existing inputs: `handover.md` and `icon_idea.jpg`.
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
| 2026-11-08 | Defer crate/event-loop/settings-format choices until environment and current APIs are verified | The handover makes Windows click handling, thread ownership, small dependencies, and compatibility selection criteria |

Add a short row for a material decision, including source links when verifying
external APIs. Revisit a choice when evidence changes; avoid speculative design.
