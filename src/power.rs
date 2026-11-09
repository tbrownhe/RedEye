//! Thread-owned continuous execution-state requests, independent of the UI.

use std::{error::Error, fmt, marker::PhantomData, rc::Rc};

const ES_CONTINUOUS: u32 = 0x8000_0000;
const ES_SYSTEM_REQUIRED: u32 = 0x0000_0001;
const ES_DISPLAY_REQUIRED: u32 = 0x0000_0002;

// Win32 EXECUTION_STATE is DWORD (32 bits), including on 64-bit Windows.
// See SetThreadExecutionState in the Windows SDK's WinBase.h / WinNT.h.
#[link(name = "kernel32")]
unsafe extern "system" {
    fn SetThreadExecutionState(flags: u32) -> u32;
}

#[derive(Debug)]
pub struct PowerError {
    flags: u32,
}

impl fmt::Display for PowerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Windows rejected SetThreadExecutionState(0x{:08X}). \
             Retry the operation or close RedEye to release its thread-owned request. \
             Check Windows power policy if this continues.",
            self.flags
        )
    }
}

impl Error for PowerError {}

/// Create and use one owner on the application's long-lived main thread.
///
/// The marker makes this type neither Send nor Sync: safe Rust cannot move or
/// share it across threads. Normal shutdown must call `clear` and handle its
/// result explicitly; a destructor could not report a failed clear reliably.
pub struct PowerRequest {
    applied_flags: u32,
    _owning_thread: PhantomData<Rc<()>>,
}

impl PowerRequest {
    pub fn new() -> Self {
        Self {
            applied_flags: ES_CONTINUOUS,
            _owning_thread: PhantomData,
        }
    }

    pub fn is_active(&self) -> bool {
        self.applied_flags & ES_SYSTEM_REQUIRED != 0
    }

    pub fn set_awake(&mut self, enabled: bool, keep_display_on: bool) -> Result<(), PowerError> {
        self.apply_with(enabled, keep_display_on, set_execution_state)
    }

    pub fn clear(&mut self) -> Result<(), PowerError> {
        self.set_awake(false, false)
    }

    // A tiny injection point lets tests exercise the same commit-after-success
    // path without changing the test runner thread's actual power requests.
    fn apply_with(
        &mut self,
        enabled: bool,
        keep_display_on: bool,
        call: impl FnOnce(u32) -> u32,
    ) -> Result<(), PowerError> {
        let flags = execution_flags(enabled, keep_display_on);
        if call(flags) == 0 {
            // This API does not document GetLastError. Do not report a stale
            // unrelated last-error value as the reason for this failure.
            return Err(PowerError { flags });
        }
        self.applied_flags = flags;
        Ok(())
    }
}

fn execution_flags(enabled: bool, keep_display_on: bool) -> u32 {
    let mut flags = ES_CONTINUOUS;
    if enabled {
        flags |= ES_SYSTEM_REQUIRED;
        if keep_display_on {
            flags |= ES_DISPLAY_REQUIRED;
        }
    }
    flags
}

fn set_execution_state(flags: u32) -> u32 {
    // SAFETY: This synchronous Kernel32 API takes/returns only a DWORD, with
    // no pointers or borrowed storage. Its ABI matches the Windows SDK. Calls
    // come from the non-Send/non-Sync owner's thread; only documented flags
    // are constructed above. The caller checks the zero failure sentinel.
    unsafe { SetThreadExecutionState(flags) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_four_flag_combinations_keep_continuous_and_gate_display() {
        assert_eq!(execution_flags(false, false), 0x8000_0000);
        assert_eq!(execution_flags(false, true), 0x8000_0000);
        assert_eq!(execution_flags(true, false), 0x8000_0001);
        assert_eq!(execution_flags(true, true), 0x8000_0003);
    }

    #[test]
    fn failed_activation_and_deactivation_preserve_last_successful_state() {
        let mut request = PowerRequest::new();
        assert!(request.apply_with(true, false, |_| 0).is_err());
        assert!(!request.is_active());

        request
            .apply_with(true, true, |flags| {
                assert_eq!(
                    flags,
                    ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED
                );
                ES_CONTINUOUS
            })
            .unwrap();
        assert!(request.is_active());
        let active_flags = request.applied_flags;

        assert!(request.apply_with(false, true, |_| 0).is_err());
        assert_eq!(request.applied_flags, active_flags);
        assert!(request.is_active());

        request.apply_with(false, true, |_| active_flags).unwrap();
        assert!(!request.is_active());
        assert_eq!(request.applied_flags, ES_CONTINUOUS);
    }

    #[test]
    fn failed_display_changes_preserve_the_previous_system_and_display_requests() {
        let mut request = PowerRequest::new();
        request.apply_with(true, false, |_| ES_CONTINUOUS).unwrap();
        assert!(request.apply_with(true, true, |_| 0).is_err());
        assert_eq!(request.applied_flags, ES_CONTINUOUS | ES_SYSTEM_REQUIRED);

        request.apply_with(true, true, |_| ES_CONTINUOUS).unwrap();
        assert!(request.apply_with(true, false, |_| 0).is_err());
        assert_eq!(
            request.applied_flags,
            ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED
        );
    }
}
