#[cfg(not(windows))]
compile_error!("RedEye requires Windows 10 or Windows 11.");

mod power;

use std::{
    error::Error,
    io::{self, BufRead, Write},
    process::ExitCode,
};

use power::PowerRequest;

fn main() -> ExitCode {
    // The main thread owns activation, the blocking input loop, and cleanup.
    // ES_CONTINUOUS maintains the request while read_line sleeps; no polling
    // or repeated keep-awake calls are necessary.
    let mut request = PowerRequest::new();
    let result = run_console(&mut request);

    // Always attempt explicit cleanup, including after activation or I/O
    // failure. Do not use process::exit or rely on Drop for this operation.
    let cleanup = request.clear();
    let mut stderr = io::stderr().lock();
    if let Err(error) = &result {
        let _ = writeln!(stderr, "RedEye: {error}");
    }
    if let Err(error) = &cleanup {
        let _ = writeln!(stderr, "RedEye: cleanup failed: {error}");
        let _ = writeln!(
            stderr,
            "Exiting now; Windows releases the request when the owning thread terminates."
        );
    } else {
        let _ = writeln!(
            io::stdout().lock(),
            "RedEye: execution-state request cleared."
        );
    }

    if result.is_ok() && cleanup.is_ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn run_console(request: &mut PowerRequest) -> Result<(), Box<dyn Error>> {
    request.set_awake(true, false)?;
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let mut output = io::stdout().lock();
    writeln!(output, "RedEye: Awake (display may turn off).")?;
    writeln!(output, "Commands: on, off, status, quit. EOF also exits.")?;
    writeln!(
        output,
        "Use quit for explicit cleanup; Ctrl+C or closing the console terminates the process."
    )?;

    let mut line = String::new();
    loop {
        output.flush()?;
        line.clear();
        if input.read_line(&mut line)? == 0 {
            return Ok(());
        }
        match line.trim() {
            "on" => {
                request.set_awake(true, false)?;
                writeln!(output, "RedEye: Awake (display may turn off).")?;
            }
            "off" => {
                request.clear()?;
                writeln!(output, "RedEye: Sleep allowed; request cleared.")?;
            }
            "status" => writeln!(
                output,
                "RedEye: {}.",
                if request.is_active() {
                    "Awake (display may turn off)"
                } else {
                    "Sleep allowed"
                }
            )?,
            "quit" => return Ok(()),
            _ => writeln!(output, "Commands: on, off, status, quit.")?,
        }
    }
}
