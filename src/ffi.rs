//! C ABI: caller owns each Session and returned string; never use one handle
//! concurrently or after free. All panics are caught before crossing the ABI.
use crate::protocol::Session;
use std::ffi::{CStr, CString, c_char};
use std::panic::{AssertUnwindSafe, catch_unwind};

fn output(value: String) -> *mut c_char {
    // Config errors are plain text, and serde may include decoded user input.
    // In particular, an escaped NUL in an unknown enum variant must remain an
    // ordinary error, not panic outside the FFI unwind boundary.
    CString::new(value.replace('\0', "\\0"))
        .expect("NUL escaped above")
        .into_raw()
}

/// # Safety
/// input must point to a valid NUL-terminated UTF-8 JSON Config. error must point
/// to writable pointer storage. A non-null error is freed with rm_string_free.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rm_new(input: *const c_char, error: *mut *mut c_char) -> *mut Session {
    if error.is_null() {
        return std::ptr::null_mut();
    }
    unsafe {
        *error = std::ptr::null_mut();
    }
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<Session, String> {
        if input.is_null() {
            return Err("null config".into());
        }
        let text = unsafe { CStr::from_ptr(input) }
            .to_str()
            .map_err(|e| e.to_string())?;
        let config = serde_json::from_str(text).map_err(|e| e.to_string())?;
        Session::new(config).map_err(|e| e.to_string())
    }));
    match result {
        Ok(Ok(session)) => Box::into_raw(Box::new(session)),
        Ok(Err(message)) => {
            unsafe {
                *error = output(message);
            }
            std::ptr::null_mut()
        }
        Err(_) => {
            unsafe {
                *error = output("engine panic".into());
            }
            std::ptr::null_mut()
        }
    }
}

/// Construct a session with an explicit outside-game pool. The legacy `rm_new`
/// JSON shape and ABI remain unchanged.
///
/// # Safety
/// `input` must be a NUL-terminated UTF-8 JSON envelope and `error` writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rm_new_with_outside(
    input: *const c_char,
    error: *mut *mut c_char,
) -> *mut Session {
    if error.is_null() {
        return std::ptr::null_mut();
    }
    unsafe {
        *error = std::ptr::null_mut();
    }
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Envelope {
        config: crate::Config,
        outside: crate::OutsideGameConfig,
    }
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<Session, String> {
        if input.is_null() {
            return Err("null config".into());
        }
        let text = unsafe { CStr::from_ptr(input) }
            .to_str()
            .map_err(|e| e.to_string())?;
        let envelope: Envelope = serde_json::from_str(text).map_err(|e| e.to_string())?;
        Session::new_with_outside(envelope.config, envelope.outside).map_err(|e| e.to_string())
    }));
    match result {
        Ok(Ok(session)) => Box::into_raw(Box::new(session)),
        Ok(Err(message)) => {
            unsafe {
                *error = output(message);
            }
            std::ptr::null_mut()
        }
        Err(_) => {
            unsafe {
                *error = output("engine panic".to_owned());
            }
            std::ptr::null_mut()
        }
    }
}

/// # Safety
/// handle must be a live, exclusively held rm_new/rm_clone handle; input must be
/// valid NUL-terminated UTF-8. Free the result with rm_string_free.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rm_request(handle: *mut Session, input: *const c_char) -> *mut c_char {
    let result = catch_unwind(AssertUnwindSafe(|| {
        if handle.is_null() || input.is_null() {
            return "{\"error\":\"null argument\"}".into();
        }
        match unsafe { CStr::from_ptr(input) }.to_str() {
            Ok(text) => unsafe { &mut *handle }.json(text),
            Err(_) => "{\"error\":\"invalid UTF-8\"}".into(),
        }
    }));
    output(result.unwrap_or_else(|_| "{\"error\":\"engine panic; discard handle\"}".into()))
}

/// # Safety
/// handle must be live and exclusively held. Returned handle has independent ownership.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rm_clone(handle: *const Session) -> *mut Session {
    if handle.is_null() {
        return std::ptr::null_mut();
    }
    catch_unwind(AssertUnwindSafe(|| {
        Box::into_raw(Box::new(unsafe { &*handle }.fork()))
    }))
    .unwrap_or(std::ptr::null_mut())
}

/// # Safety
/// handle must be null or an unfreed pointer returned by rm_new/rm_clone.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rm_free(handle: *mut Session) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

/// # Safety
/// value must be null or an unfreed pointer returned by this library.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rm_string_free(value: *mut c_char) {
    if !value.is_null() {
        unsafe {
            drop(CString::from_raw(value));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rm_abi_version() -> u32 {
    1
}
