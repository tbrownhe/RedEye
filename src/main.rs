#[cfg(not(windows))]
compile_error!("RedEye requires Windows 10 or Windows 11.");

mod instance;
mod power;
mod settings;
mod tray;

use std::{io::Write, process::ExitCode};

use power::PowerRequest;

fn main() -> ExitCode {
    // Keep the session/user mutex handle alive until after power cleanup.
    // A duplicate returns before changing power, reading settings, or adding UI.
    let _instance = match instance::acquire() {
        Ok(Some(handle)) => handle,
        Ok(None) => return ExitCode::SUCCESS,
        Err(error) => {
            tray::report_error(&error);
            return ExitCode::FAILURE;
        }
    };
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
    let keep_display_on = match settings::load_display() {
        Ok(preference) => preference,
        Err(error) => {
            tray::report_notice(&format!(
                "{error}\n\nFor this launch, Keep display on defaults to off. Change that menu option to save a valid preference."
            ));
            false
        }
    };
    request
        .set_awake(true, keep_display_on)
        .map_err(|e| e.to_string())?;
    let mut tray = tray::Tray::new(request.is_active(), keep_display_on)?;
    let _ = writeln!(
        std::io::stdout(),
        "RedEye: {}. Left-click toggles; right-click opens the menu.",
        if request.is_active() {
            if keep_display_on {
                "Awake (display kept on)"
            } else {
                "Awake (display may turn off)"
            }
        } else {
            "Sleep allowed"
        }
    );
    let result = run_events(request, &mut tray, keep_display_on);
    // Normal cleanup is explicit so failures can be reported. Drop provides a
    // fallback for partially initialized resources or an unexpected unwind.
    let cleanup = tray.close();
    match (result, cleanup) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
        (Err(error), Err(cleanup)) => Err(format!("{error}\nTray cleanup: {cleanup}")),
    }
}

fn run_events(
    request: &mut PowerRequest,
    tray: &mut tray::Tray,
    mut keep_display_on: bool,
) -> Result<(), String> {
    // The display option is a preference, distinct from the effective awake
    // state. Only this preference is saved; every launch starts active.
    loop {
        let mut event = tray.next_event()?;
        if matches!(event, tray::Event::OpenMenu) {
            refresh_startup(tray)?;
            let Some(selected) = tray.show_menu()? else {
                continue;
            };
            event = selected;
        }
        let save_display = matches!(event, tray::Event::ToggleDisplay);
        let (active, display) = match event {
            tray::Event::ToggleAwake => (!request.is_active(), keep_display_on),
            tray::Event::ToggleDisplay => (request.is_active(), !keep_display_on),
            tray::Event::ToggleStartup => {
                match settings::startup_enabled() {
                    Ok(enabled) => {
                        let result = settings::set_startup(!enabled);
                        refresh_startup(tray)?;
                        if let Err(error) = result {
                            tray::report_notice(&error);
                        }
                    }
                    Err(error) => {
                        tray.refresh_startup(None)?;
                        tray::report_notice(&format!(
                            "{error}\n\nLaunch at sign-in is unavailable until its registration can be read."
                        ));
                    }
                }
                continue;
            }
            tray::Event::TaskbarCreated => {
                tray.restore(request.is_active())?;
                continue;
            }
            tray::Event::OpenMenu => continue,
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
        if save_display && let Err(error) = settings::save_display(keep_display_on) {
            tray::report_notice(&format!(
                "{error}\n\nThe current display preference is applied for this run, but could not be saved. The next launch will use the previously saved value. Retry by changing Keep display on again."
            ));
        }
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

fn refresh_startup(tray: &mut tray::Tray) -> Result<(), String> {
    match settings::startup_enabled() {
        Ok(enabled) => tray.refresh_startup(Some(enabled)),
        Err(error) => {
            tray.refresh_startup(None)?;
            tray::report_notice(&format!(
                "{error}\n\nLaunch at sign-in is unavailable until its registration can be read. Its previous checkmark is retained."
            ));
            Ok(())
        }
    }
}
