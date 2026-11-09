//! One instance per Windows user in each interactive session.

use std::{
    mem::size_of,
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
    ptr, slice,
};

use windows_sys::Win32::{
    Foundation::{ERROR_ALREADY_EXISTS, ERROR_INSUFFICIENT_BUFFER, GetLastError, LocalFree},
    Security::{
        Authorization::ConvertSidToStringSidW, GetTokenInformation, TOKEN_QUERY, TOKEN_USER,
        TokenUser,
    },
    System::Threading::{CreateMutexW, GetCurrentProcess, OpenProcessToken},
};

/// The returned handle keeps the named object alive; no mutex lock is needed.
/// None means another instance already holds the object in this user/session.
pub fn acquire() -> Result<Option<OwnedHandle>, String> {
    let name: Vec<u16> = format!("Local\\RedEye.{}", current_user_sid()?)
        .encode_utf16()
        .chain(Some(0))
        .collect();
    // SAFETY: name is NUL-terminated and alive for this synchronous call.
    // Null attributes use the user's default ACL and prevent inheritance.
    let raw = unsafe { CreateMutexW(ptr::null(), 0, name.as_ptr()) };
    // The existing-object indication must be read before another Win32 call.
    let status = unsafe { GetLastError() };
    if raw.is_null() {
        return Err(format!(
            "Cannot create the RedEye session mutex: {}",
            std::io::Error::from_raw_os_error(status as i32)
        ));
    }
    // SAFETY: CreateMutexW returned a unique, valid owned handle. OwnedHandle
    // closes it on every return path, including the duplicate launch path.
    let handle = unsafe { OwnedHandle::from_raw_handle(raw) };
    if status == ERROR_ALREADY_EXISTS {
        Ok(None)
    } else {
        Ok(Some(handle))
    }
}

fn current_user_sid() -> Result<String, String> {
    let mut raw_token = ptr::null_mut();
    // SAFETY: pseudo process handle is valid and never closed. raw_token is
    // writable; TOKEN_QUERY only requests read access to this process's token.
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut raw_token) } == 0 {
        return Err(os_error("OpenProcessToken"));
    }
    // SAFETY: OpenProcessToken succeeded and transferred ownership.
    let token = unsafe { OwnedHandle::from_raw_handle(raw_token) };
    let mut needed = 0;
    // SAFETY: The documented sizing call supplies no output buffer and zero
    // capacity. It must fail with insufficient buffer and fill needed.
    let sized = unsafe {
        GetTokenInformation(
            token.as_raw_handle(),
            TokenUser,
            ptr::null_mut(),
            0,
            &mut needed,
        )
    };
    let status = unsafe { GetLastError() };
    if sized != 0
        || status != ERROR_INSUFFICIENT_BUFFER
        || (needed as usize) < size_of::<TOKEN_USER>()
    {
        return Err(format!(
            "Cannot size the Windows user token: {}",
            std::io::Error::from_raw_os_error(status as i32)
        ));
    }
    // TOKEN_USER contains pointers, so Vec<u8> would not guarantee alignment.
    let mut storage = vec![0usize; (needed as usize).div_ceil(size_of::<usize>())];
    let capacity = needed;
    // SAFETY: storage is pointer-aligned, writable, and at least capacity bytes.
    if unsafe {
        GetTokenInformation(
            token.as_raw_handle(),
            TokenUser,
            storage.as_mut_ptr().cast(),
            capacity,
            &mut needed,
        )
    } == 0
    {
        return Err(os_error("GetTokenInformation"));
    }
    if needed < size_of::<TOKEN_USER>() as u32 || needed > capacity {
        return Err("Windows returned an invalid user-token size.".into());
    }
    // SAFETY: Successful TokenUser query initialized the aligned TOKEN_USER
    // structure and its SID inside storage. storage remains alive through conversion.
    let user = unsafe { &*storage.as_ptr().cast::<TOKEN_USER>() };
    let mut sid_text = ptr::null_mut();
    if unsafe { ConvertSidToStringSidW(user.User.Sid, &mut sid_text) } == 0 {
        return Err(os_error("ConvertSidToStringSidW"));
    }
    // SAFETY: Success guarantees a NUL-terminated allocated UTF-16 SID string.
    let result = unsafe {
        let mut length = 0;
        while *sid_text.add(length) != 0 {
            length += 1;
        }
        String::from_utf16(slice::from_raw_parts(sid_text, length))
    };
    // SAFETY: This exact buffer was allocated by ConvertSidToStringSidW.
    if !unsafe { LocalFree(sid_text.cast()) }.is_null() {
        return Err(os_error("LocalFree SID string"));
    }
    result.map_err(|_| "Windows returned an invalid user SID string.".into())
}

fn os_error(operation: &str) -> String {
    format!("{operation} failed: {}", std::io::Error::last_os_error())
}
