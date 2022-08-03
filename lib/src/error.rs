use crate::sqlite3ext::sqlite3_api_routines;
use std::{
    ffi::CString,
    os::raw::{c_char, c_int},
    ptr::copy_nonoverlapping,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SheetError {
    #[error("No ID is provided")]
    NoId,
    #[error("No sheet name is provided")]
    NoSheet,
    #[error("Invalid range is provided")]
    InvalidRange,
    #[error("Unknown option is provided")]
    UnknownOption,
    #[error(transparent)]
    Api(#[from] google_sheets_api::error::Error),
}

impl From<SheetError> for String {
    fn from(s: SheetError) -> Self {
        s.to_string()
    }
}

pub unsafe fn error_to_sqlite3_string(
    api: *mut sqlite3_api_routines,
    err: impl Into<String>,
) -> Option<*mut c_char> {
    let cstr = CString::new(err.into()).ok()?;
    let len = cstr.as_bytes_with_nul().len();

    let ptr = ((*api).malloc.unwrap())(len as c_int) as *mut c_char;
    if !ptr.is_null() {
        copy_nonoverlapping(cstr.as_ptr(), ptr, len);
        Some(ptr)
    } else {
        None
    }
}
