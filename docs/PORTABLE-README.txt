RedEye {{VERSION}} - Windows 10/11 x64 portable release candidate
Source commit: {{COMMIT}}
Repository and validation evidence: https://github.com/tbrownhe/RedEye

INSTALL
Copy RedEye.exe to a stable folder, for example %LOCALAPPDATA%\RedEye,
and launch it as a standard user. No installer, Cargo, external icon assets,
or separate Visual C++ runtime is required. The executable is unsigned.

USE
RedEye starts active; the display is allowed off unless you saved another
preference. Its tray icon may be inside the notification-area overflow.
Left-click toggles awake state. Right-click opens the native menu:
- Stay awake: prevent automatic idle sleep.
- Keep display on: retain this preference, applied only while active.
- Launch at sign-in: opt in to per-user startup from this stable copy.
- Exit: remove the icon, explicitly clear the request, and close RedEye.
A duplicate launch exits; the existing instance continues running.

LIMITATIONS AND VALIDATION
RedEye does not prevent locking, explicit sleep/shutdown, policy enforcement,
or remote-session inactivity. It never simulates keyboard or mouse input.
The icon uses system DPI at launch; restart after changing display scaling.
Windows 11 core behavior was checked during development. Windows 10,
actual sign-in launch, actual Explorer restart, cross-session isolation,
multiple DPI settings, and lock/explicit-sleep behavior remain unverified.
Automated builds and checksums do not establish native Windows behavior.
See docs/VALIDATION.md in the source repository for exact evidence and steps.

SETTINGS AND REMOVAL
Only the RedEye value under HKCU\Software\Microsoft\Windows\CurrentVersion\Run
is changed for startup. Known Cargo/temporary paths cannot be registered.
KeepDisplayOn is a DWORD under HKCU\Software\RedEye. Awake state is not saved.
To remove: disable Launch at sign-in, choose Exit, then delete RedEye.exe.
Optionally remove only the KeepDisplayOn value. No service or scheduled task
is installed. Windows startup controls and policy can override registration.

BUILD AND VERIFY
Source builds require Rust 1.99.1 x64 MSVC, C++ tools, and the Windows SDK:
  cargo build --release --locked
  powershell -NoProfile -ExecutionPolicy Bypass -File tools/package-release.ps1
The ZIP's SHA256SUMS.txt covers RedEye.exe and this README.txt.
The separately downloaded SHA256SUMS.txt covers RedEye.exe and the ZIP.
Use PowerShell Get-FileHash -Algorithm SHA256 to compare the listed hashes.
Checksums detect changed downloads; they are not a code-signing certificate.
