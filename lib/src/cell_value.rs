use crate::sqlite3ext::{sqlite3_api_routines, sqlite3_context};
use google_sheets_api::CellData;
use std::{
    ffi::CString,
    os::raw::{c_char, c_int, c_void},
};

#[derive(Debug)]
enum CellValue {
    Str(String),
    Float(f64),
    Int(i64),
    Empty,
}

pub unsafe fn yield_cell_value(
    p_context: *mut sqlite3_context,
    api: *mut sqlite3_api_routines,
    value: Option<&CellData>,
) {
    match parse_value(value) {
        CellValue::Str(s) => {
            let (len, raw) = to_raw_string(s);
            ((*api).result_text.unwrap())(p_context, raw, len as c_int, Some(destructor))
        }
        CellValue::Float(f) => ((*api).result_double.unwrap())(p_context, f),
        CellValue::Int(i) => ((*api).result_int64.unwrap())(p_context, i),
        CellValue::Empty => ((*api).result_null.unwrap())(p_context),
    }
}

fn parse_value(value: Option<&CellData>) -> CellValue {
    if let Some(v) = value {
        if let (Some(formatted_str), Some(effective_value)) =
            (&v.formatted_value, &v.effective_value)
        {
            // if effective value is a string, of course it's string
            if let Some(str) = &effective_value.string_value {
                return CellValue::Str(str.to_string());
            }

            if let Some(num) = &effective_value.number_value {
                // if effective value is a number, but (1) it couldn't be parsed as i64 or f64, and
                // (2) formatted value doesn't contain '%', it might be a Dates, Times, or DateTimes.
                // Since it's difficult to parse google_sheets4::api::NumberFormat into
                // chrono::format::strftime, returns formatted string as a cell representation
                if (formatted_str.parse::<i64>().is_err() || formatted_str.parse::<f64>().is_err())
                    && !formatted_str.contains('%')
                {
                    return CellValue::Str(formatted_str.to_string());
                }

                // number value is always f64, but if formatted string can be parse as i64, returns
                // it instead because it's intuitive
                if let Ok(v) = formatted_str.parse::<i64>() {
                    return CellValue::Int(v);
                }

                return CellValue::Float(*num);
            }

            return CellValue::Str(formatted_str.to_string());
        } else {
            return CellValue::Empty;
        }
    }
    CellValue::Empty
}

fn to_raw_string(s: String) -> (usize, *mut c_char) {
    let cstr = CString::new(s.as_str().as_bytes()).unwrap();
    let len = cstr.as_bytes().len();
    let raw = cstr.into_raw();

    (len, raw)
}

unsafe extern "C" fn destructor(raw: *mut c_void) {
    drop(CString::from_raw(raw as *mut c_char));
}
