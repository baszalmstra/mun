//! Exposes error reporting using the C ABI.

use std::{ffi::CString, os::raw::c_char, ptr};
use std::ffi::CStr;

#[repr(C)]
#[derive(Clone, Copy)]
/// A C-style handle to an error message.
///
/// If the handle contains a non-null pointer, an error occurred.
/// cbindgen:field-names=[error_string]
pub struct ErrorHandle(pub *const c_char);

impl ErrorHandle {
    /// Constructs an `ErrorHandle` from the specified error message.
    pub fn new<T: Into<Vec<u8>>>(error_message: T) -> Self {
        let error_message = CString::new(error_message).expect("Invalid error message");
        Self(CString::into_raw(error_message))
    }

    /// Returns true if this error handle doesnt actually contain any error.
    pub fn is_ok(&self) -> bool {
        self.0.is_null()
    }

    /// Returns true if this error handle contains an error
    pub fn is_err(&self) -> bool {
        !self.0.is_null()
    }

    /// Returns the error associated with this instance or `None` if there is no error.
    ///
    /// # Safety
    ///
    /// If the error contained in this handle has previously been deallocated the data may have been
    /// corrupted.
    pub unsafe fn err(&self) -> Option<&CStr> {
        if self.is_err() {
            Some(CStr::from_ptr(self.0))
        } else {
            None
        }
    }
}

impl Default for ErrorHandle {
    fn default() -> Self {
        Self(ptr::null())
    }
}

impl<T: Into<Vec<u8>>> From<T> for ErrorHandle {
    fn from(bytes: T) -> Self {
        ErrorHandle::new(bytes)
    }
}

/// Destructs the error message corresponding to the specified handle.
///
/// # Safety
///
/// Only call this function on an ErrorHandle once.
#[no_mangle]
pub unsafe extern "C" fn mun_error_destroy(error: ErrorHandle) {
    if !error.0.is_null() {
        let _drop = CString::from_raw(error.0 as *mut c_char);
    }
}

#[macro_export]
macro_rules! mun_error_try {
    ($expr:expr $(,)?) => {
        match $expr {
            Ok(val) => val,
            Err(err) => {
                return ErrorHandle::from(err);
            }
        }
    };
}

#[macro_export]
macro_rules! try_deref_mut {
    ($expr:expr $(,)?) => {
        match ($expr).as_mut() {
            Some(val) => val,
            None => {
                return ErrorHandle::new(concat!(stringify!($expr), " must not be null"));
            }
        }
    }
}

#[macro_export]
macro_rules! assert_error {
    ($expr:expr $(,)?) => {
        let err = $expr;
        assert!(err.is_err());
        unsafe { $crate::mun_string_destroy(err.0) };
    }
}
