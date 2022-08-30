use crate::{
    cell_value::yield_cell_value,
    error::error_to_sqlite3_string,
    module_argument::collect_options_from_args,
    reader::SheetReader,
    sheet::Sheet,
    sqlite3ext::{
        sqlite3, sqlite3_api_routines, sqlite3_context, sqlite3_index_info, sqlite3_int64,
        sqlite3_module, sqlite3_value, sqlite3_vtab, sqlite3_vtab_cursor, SQLITE_ERROR, SQLITE_OK,
        SQLITE_OK_LOAD_PERMANENTLY,
    },
};
use google_sheets_api::client::GoogleSheetsReadOnlyClient;
use std::{
    env,
    ffi::c_void,
    ffi::CString,
    os::raw::{c_char, c_int, c_longlong},
    sync::{Arc, Mutex},
};

#[no_mangle]
static mut SQLITE3_API: *mut sqlite3_api_routines = std::ptr::null_mut();

#[repr(C)]
pub struct Module {
    // must be at the beginning
    base: sqlite3_module,
    name: &'static [u8],
}

const GSQLITE_MODULE: Module = Module {
    base: sqlite3_module {
        iVersion: 0,
        xCreate: Some(gsqlite_create),
        xConnect: Some(gsqlite_connect),
        xBestIndex: Some(gsqlite_best_index),
        xDisconnect: Some(gsqlite_disconnect),
        xDestroy: Some(gsqlite_destroy),
        xOpen: Some(gsqlite_open),
        xClose: Some(gsqlite_close),
        xFilter: Some(gsqlite_filter),
        xNext: Some(gsqlite_next),
        xEof: Some(gsqlite_eof),
        xColumn: Some(gsqlite_column),
        xRowid: Some(gsqlite_rowid),
        xUpdate: None,
        xBegin: None,
        xSync: None,
        xCommit: None,
        xRollback: None,
        xFindFunction: None,
        xRename: None,
        xSavepoint: None,
        xRelease: None,
        xRollbackTo: None,
        xShadowName: None,
    },
    name: b"gsqlite\0",
};

#[repr(C)]
pub struct VirtualTable {
    // must be at the beginning
    pub base: sqlite3_vtab,
    pub sheet: Arc<Mutex<Sheet>>,
}

#[repr(C)]
pub struct VirtualCursor {
    // must be at the beginning
    pub base: sqlite3_vtab_cursor,
    pub reader: Arc<Mutex<SheetReader>>,
}

#[no_mangle]
unsafe extern "C" fn register_module(
    db: *mut sqlite3,
    pz_err_msg: *mut *mut c_char,
    p_api: *mut sqlite3_api_routines,
) -> c_int {
    let result = ((*p_api).create_module.unwrap())(
        db,
        GSQLITE_MODULE.name.as_ptr() as *const c_char,
        &GSQLITE_MODULE as *const Module as *const sqlite3_module,
        std::ptr::null_mut(),
    );

    match result {
        SQLITE_OK => SQLITE_OK_LOAD_PERMANENTLY,
        _ => {
            let err = format!("Failed to create module, status: {}", result);
            if let Some(ptr) = error_to_sqlite3_string(SQLITE3_API, err) {
                *pz_err_msg = ptr;
            }
            SQLITE_ERROR
        }
    }
}

#[no_mangle]
unsafe extern "C" fn sqlite3_gsqlite_init(
    db: *mut sqlite3,
    pz_err_msg: *mut *mut c_char,
    p_api: *mut sqlite3_api_routines,
) -> c_int {
    SQLITE3_API = p_api;

    let result = register_module(db, pz_err_msg, p_api);
    match result {
        SQLITE_OK => {
            let result = ((*p_api).auto_extension.unwrap())(Some(std::mem::transmute(
                register_module as *const (),
            )));
            if result != SQLITE_OK {
                return result;
            }
        }
        _ => return result,
    }

    SQLITE_OK_LOAD_PERMANENTLY
}

#[no_mangle]
unsafe extern "C" fn gsqlite_create(
    db: *mut sqlite3,
    _p_aux: *mut c_void,
    argc: c_int,
    argv: *const *const c_char,
    pp_vtab: *mut *mut sqlite3_vtab,
    pz_err: *mut *mut c_char,
) -> c_int {
    let client_id = match env::var("LIBGSQLITE_GOOGLE_CLIENT_ID") {
        Ok(v) => v,
        Err(_) => panic!("Environment variable LIBGSQLITE_GOOGLE_CLIENT_ID is not set"),
    };
    let client_secret = match env::var("LIBGSQLITE_GOOGLE_CLIENT_SECRET") {
        Ok(v) => v,
        Err(_) => panic!("Environment variable LIBGSQLITE_GOOGLE_CLIENT_SECRET is not set"),
    };

    match collect_options_from_args(argc, argv) {
        Ok((id, sheet, range)) => {
            let mut sheet = Sheet::builder()
                .client(
                    GoogleSheetsReadOnlyClient::builder()
                        .client_id(client_id)
                        .client_secret(client_secret)
                        .cache_access_token(true)
                        .build(),
                )
                .id(id)
                .sheet(sheet)
                .range(range)
                .build();

            match sheet.open() {
                Ok(_) => {
                    let result = declare_table(db, SQLITE3_API, sheet.get_columns());
                    let p_new = Box::new(VirtualTable {
                        base: sqlite3_vtab {
                            pModule: std::ptr::null_mut(),
                            nRef: 0,
                            zErrMsg: std::ptr::null_mut(),
                        },
                        sheet: Arc::new(Mutex::new(sheet)),
                    });
                    *pp_vtab = Box::into_raw(p_new) as *mut sqlite3_vtab;
                    result
                }
                Err(err) => {
                    if let Some(ptr) = error_to_sqlite3_string(SQLITE3_API, err) {
                        *pz_err = ptr;
                    }
                    SQLITE_ERROR
                }
            }
        }
        Err(_) => SQLITE_ERROR,
    }
}

#[no_mangle]
unsafe extern "C" fn gsqlite_connect(
    db: *mut sqlite3,
    p_aux: *mut c_void,
    argc: c_int,
    argv: *const *const c_char,
    pp_vtab: *mut *mut sqlite3_vtab,
    pz_err: *mut *mut c_char,
) -> c_int {
    gsqlite_create(db, p_aux, argc, argv, pp_vtab, pz_err)
}

#[no_mangle]
unsafe extern "C" fn gsqlite_best_index(
    _p_vtab: *mut sqlite3_vtab,
    _arg1: *mut sqlite3_index_info,
) -> c_int {
    SQLITE_OK
}

#[no_mangle]
unsafe extern "C" fn gsqlite_disconnect(p_vtab: *mut sqlite3_vtab) -> c_int {
    gsqlite_destroy(p_vtab)
}

#[no_mangle]
unsafe extern "C" fn gsqlite_destroy(p_vtab: *mut sqlite3_vtab) -> c_int {
    if !p_vtab.is_null() {
        let table = Box::from_raw(p_vtab as *mut VirtualTable);
        drop(table);
    }

    SQLITE_OK
}

#[no_mangle]
unsafe extern "C" fn gsqlite_open(
    p_vtab: *mut sqlite3_vtab,
    pp_cursor: *mut *mut sqlite3_vtab_cursor,
) -> c_int {
    let table = &mut *(p_vtab as *mut VirtualTable);
    let sheet = Arc::clone(&table.sheet);
    let mut lock = sheet.lock().unwrap();
    let reader = lock.get_reader();

    let cursor = Box::new(VirtualCursor {
        base: sqlite3_vtab_cursor { pVtab: p_vtab },
        reader: Arc::new(Mutex::new(reader)),
    });
    *pp_cursor = Box::into_raw(cursor) as _;

    SQLITE_OK
}

#[no_mangle]
unsafe extern "C" fn gsqlite_close(p_cursor: *mut sqlite3_vtab_cursor) -> c_int {
    if !p_cursor.is_null() {
        let cursor = Box::from_raw(p_cursor as *mut VirtualCursor);
        drop(cursor);
    }

    SQLITE_OK
}

#[no_mangle]
unsafe extern "C" fn gsqlite_filter(
    _arg1: *mut sqlite3_vtab_cursor,
    _idx_num: c_int,
    _idx_str: *const c_char,
    _argc: c_int,
    _argv: *mut *mut sqlite3_value,
) -> c_int {
    SQLITE_OK
}

#[no_mangle]
unsafe extern "C" fn gsqlite_next(p_cursor: *mut sqlite3_vtab_cursor) -> c_int {
    let cursor = &mut *(p_cursor as *mut VirtualCursor);
    let lock = Arc::clone(&cursor.reader);
    let mut reader = lock.lock().unwrap();

    reader.move_next();

    SQLITE_OK
}

#[no_mangle]
unsafe extern "C" fn gsqlite_eof(p_cursor: *mut sqlite3_vtab_cursor) -> c_int {
    let cursor = &mut *(p_cursor as *mut VirtualCursor);
    let lock = Arc::clone(&cursor.reader);
    let reader = lock.lock().unwrap();

    if reader.has_value() {
        0
    } else {
        1
    }
}

#[no_mangle]
unsafe extern "C" fn gsqlite_column(
    p_cursor: *mut sqlite3_vtab_cursor,
    p_context: *mut sqlite3_context,
    column: c_int,
) -> c_int {
    let cursor = &mut *(p_cursor as *mut VirtualCursor);
    let lock = Arc::clone(&cursor.reader);
    let reader = lock.lock().unwrap();

    yield_cell_value(p_context, SQLITE3_API, reader.get_value(column as usize));

    SQLITE_OK
}

#[no_mangle]
unsafe extern "C" fn gsqlite_rowid(
    p_cursor: *mut sqlite3_vtab_cursor,
    p_rowid: *mut sqlite3_int64,
) -> c_int {
    let cursor = &mut *(p_cursor as *mut VirtualCursor);
    let lock = Arc::clone(&cursor.reader);
    let reader = lock.lock().unwrap();

    *p_rowid = reader.get_rowid() as c_longlong;

    SQLITE_OK
}

unsafe fn declare_table(
    db: *mut sqlite3,
    api: *mut sqlite3_api_routines,
    columns: Vec<String>,
) -> c_int {
    ((*api).declare_vtab.unwrap())(db, create_declare_table_statement(columns).as_ptr() as _)
}

fn create_declare_table_statement(columns: Vec<String>) -> CString {
    CString::new(format!("CREATE TABLE sheet({})", columns.join(", "))).unwrap()
}

#[cfg(test)]
mod tests {
    use crate::module::create_declare_table_statement;
    use rusqlite::{Connection, LoadExtensionGuard};
    use std::{
        env,
        ffi::CString,
        {error::Error, path::PathBuf},
    };

    #[derive(Debug, PartialEq)]
    struct Employee {
        employee_number: i32,
        first_name: String,
        last_name: String,
        department: String,
    }

    fn load_my_extension(conn: &Connection) -> rusqlite::Result<()> {
        let path_buf: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "..",
            "target",
            "debug",
            "libgsqlite",
        ]
        .iter()
        .collect();

        unsafe {
            let _guard = LoadExtensionGuard::new(conn)?;
            conn.load_extension(path_buf.as_path().as_os_str(), None)
        }
    }

    #[test]
    fn test_extension() -> Result<(), Box<dyn Error>> {
        let conn = Connection::open_in_memory()?;
        load_my_extension(&conn)?;

        let id = match env::var("LIBGSQLITE_GOOGLE_CLIENT_ID") {
            Ok(v) => v,
            Err(_) => panic!("Environment variable LIBGSQLITE_GOOGLE_CLIENT_ID is not set"),
        };
        let sheet = match env::var("LIBGSQLITE_GOOGLE_CLIENT_TEST_SHEET") {
            Ok(v) => v,
            Err(_) => panic!("Environment variable LIBGSQLITE_GOOGLE_CLIENT_TEST_SHEET is not set"),
        };
        let range = match env::var("LIBGSQLITE_GOOGLE_CLIENT_TEST_RANGE") {
            Ok(v) => v,
            Err(_) => panic!("Environment variable LIBGSQLITE_GOOGLE_CLIENT_TEST_RANGE is not set"),
        };
        conn.execute(
            format!(
                r#"CREATE VIRTUAL TABLE employees USING gsqlite(ID '{}', SHEET '{}', RANGE '{}');"#,
                id, sheet, range
            )
            .as_str(),
            (),
        )?;

        let mut employees = Vec::new();
        let mut stmt = conn.prepare("SELECT * FROM employees WHERE D LIKE 'E%'")?;
        let result = stmt.query_map([], |row| {
            Ok(Employee {
                employee_number: row.get(0)?,
                first_name: row.get(1)?,
                last_name: row.get(2)?,
                department: row.get(3)?,
            })
        })?;
        for employee in result {
            employees.push(employee.unwrap());
        }

        assert_eq!(employees.len(), 2);
        assert_eq!(
            employees[0],
            Employee {
                employee_number: 4,
                first_name: "John".to_string(),
                last_name: "Beyer".to_string(),
                department: "E01".to_string()
            }
        );
        assert_eq!(
            employees[1],
            Employee {
                employee_number: 6,
                first_name: "Eva".to_string(),
                last_name: "Pulaski".to_string(),
                department: "E01".to_string()
            }
        );
        Ok(())
    }

    #[test]
    fn test_create_declare_table_statement() {
        assert_eq!(
            CString::new("CREATE TABLE sheet(A, B, C)").unwrap(),
            create_declare_table_statement(
                vec!["A", "B", "C"]
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect()
            )
        )
    }
}
