use crate::{
    error::SheetError,
    error::SheetError::{InvalidRange, NoId, NoSheet, UnknownOption},
    range::Range,
};
use regex::Regex;
use std::{
    ffi::CStr,
    os::raw::{c_char, c_int},
};

enum ModuleArgument {
    Id(String),
    Sheet(String),
    Range(Range),
}

pub unsafe fn collect_options_from_args(
    argc: c_int,
    argv: *const *const c_char,
) -> Result<(String, String, Range), SheetError> {
    let mut id = "".to_string();
    let mut sheet = "".to_string();
    let mut range = Range {
        c1: "A".to_string(),
        r1: 0,
        c2: "A".to_string(),
        r2: 0,
    };

    for arg in collect_strings_from_raw(argc as usize, argv) {
        if let Ok(option) = parse_option(arg.as_str()) {
            match option {
                ModuleArgument::Id(i) => id = i.to_string(),
                ModuleArgument::Sheet(s) => sheet = s.to_string(),
                ModuleArgument::Range(r) => range = r,
            }
        }
    }

    if id.is_empty() {
        return Err(NoId);
    }
    if sheet.is_empty() {
        return Err(NoSheet);
    }
    if range.r1 == 0 || range.r2 == 0 {
        return Err(InvalidRange);
    }

    Ok((id, sheet, range))
}

unsafe fn collect_strings_from_raw(n: usize, args: *const *const c_char) -> Vec<String> {
    let mut vec = Vec::with_capacity(n);

    let args = args as *mut *const c_char;
    for i in 0..n {
        let arg = *(args.add(i));
        let s = read_string_from_raw(arg);
        vec.push(s);
    }

    vec
}

unsafe fn read_string_from_raw(raw: *const c_char) -> String {
    let cstr = CStr::from_ptr(raw);
    cstr.to_str().unwrap_or_default().to_string()
}

fn parse_option(input: &str) -> Result<ModuleArgument, SheetError> {
    if let Ok(re) = Regex::new(r#"(?i)^(ID|SHEET|RANGE)\s+['"]([^'"]+)['"]$"#) {
        if let Some(cap) = re.captures(input) {
            return match cap[1].to_lowercase().as_str() {
                "id" => Ok(ModuleArgument::Id(cap[2].into())),
                "sheet" => Ok(ModuleArgument::Sheet(cap[2].into())),
                "range" => Ok(ModuleArgument::Range(cap[2].into())),
                _ => Err(UnknownOption),
            };
        }
    }

    Err(UnknownOption)
}

#[cfg(test)]
mod tests {
    use crate::module_argument::collect_options_from_args;
    use std::ffi::CStr;

    #[test]
    fn test_collect_options_from_args() {
        unsafe {
            let mut v = Vec::with_capacity(4);
            v.push(CStr::from_bytes_with_nul(b"id 'some_random_id'\0").unwrap());
            v.push(CStr::from_bytes_with_nul(b"SHEET \"JP\"\0").unwrap());
            v.push(CStr::from_bytes_with_nul(b"RANGE 'A2:F5'\0").unwrap());

            let out = v.into_iter().map(|s| s.as_ptr()).collect::<Vec<_>>();

            assert_eq!(
                (
                    "some_random_id".to_string(),
                    "JP".to_string(),
                    "A2:F5".into()
                ),
                collect_options_from_args(3, out.as_ptr()).unwrap()
            )
        }
    }
}
