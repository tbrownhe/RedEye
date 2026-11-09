#[cfg(not(windows))]
compile_error!("RedEye requires Windows 10 or Windows 11.");

mod power;
mod tray;

use std::{io::Write, process::ExitCode};

use power::PowerRequest;

fn main() -> ExitCode {
    // The same main thread owns activation, the blocking native event loop,
    // and explicit cleanup. ES_CONTINUOUS requires no keep-awake polling.
    let mut request = PowerRequest::new();
    let result = run(&mut request);
    let cleanup = request.clear();

    let mut failed = false;
    if let Err(error) = result {
        tray::report_error(&error);
        failed = true;
    }
    if let Err(error) = cleanup {
        tray::report_error(&format!(
            "Cleanup failed: {error}\nWindows releases the request when RedEye's owning thread exits."
        ));
        failed = true;
    } else {
        let _ = writeln!(
            std::io::stdout(),
            "RedEye: execution-state request cleared."
        );
    }

    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn run(request: &mut PowerRequest) -> Result<(), String> {
    request.set_awake(true, false).map_err(|e| e.to_string())?;
    let mut tray = tray::Tray::new(request.is_active(), false)?;
    let _ = writeln!(
        std::io::stdout(),
        "RedEye: {}. Left-click toggles; right-click opens the menu.",
        if request.is_active() {
            "Awake (display may turn off)"
        } else {
            "Sleep allowed"
        }
    );
    let result = run_events(request, &mut tray);
    // Normal cleanup is explicit so failures can be reported. Drop provides a
    // fallback for partially initialized resources or an unexpected unwind.
    let cleanup = tray.close();
    match (result, cleanup) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
        (Err(error), Err(cleanup)) => Err(format!("{error}\nTray cleanup: {cleanup}")),
    }
}

fn run_events(request: &mut PowerRequest, tray: &mut tray::Tray) -> Result<(), String> {
    // The display option is a preference, distinct from the effective awake
    // state. Milestone 3 keeps it in memory; every launch starts with it off.
    let mut keep_display_on = false;
    loop {
        let (active, display) = match tray.next_event()? {
            tray::Event::ToggleAwake => (!request.is_active(), keep_display_on),
            tray::Event::ToggleDisplay => (request.is_active(), !keep_display_on),
            tray::Event::Exit => return Ok(()),
        };
        // `?` here would exit on a recoverable power failure. Instead retain
        // the last effective state and leave every visual unchanged on error.
        if let Err(error) = request.set_awake(active, display) {
            tray::report_transition_error(&error.to_string());
            continue;
        }
        keep_display_on = display;
        // If rendering only partly succeeds, return a fatal error. Main will
        // remove the tray icon and explicitly clear power before reporting it.
        tray.update(request.is_active(), keep_display_on)?;
        let _ = writeln!(
            std::io::stdout(),
            "RedEye: {}; keep display on preference: {}.",
            if request.is_active() {
                "Awake"
            } else {
                "Sleep allowed"
            },
            keep_display_on
        );
    }
}
