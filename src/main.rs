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
    let mut tray = tray::Tray::new()?;
    let _ = writeln!(
        std::io::stdout(),
        "RedEye: {}. Right-click the tray icon for Exit. Left-click is diagnostic only in milestone 2.",
        if request.is_active() {
            "Awake (display may turn off)"
        } else {
            "Sleep allowed"
        }
    );
    let result = tray.run();
    // Normal cleanup is explicit so failures can be reported. Drop provides a
    // fallback for partially initialized resources or an unexpected unwind.
    let cleanup = tray.close();
    match (result, cleanup) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
        (Err(error), Err(cleanup)) => Err(format!("{error}\nTray cleanup: {cleanup}")),
    }
}
