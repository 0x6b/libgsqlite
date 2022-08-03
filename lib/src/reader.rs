use google_sheets_api::{CellData, RowData};

pub struct SheetReader {
    rows: Vec<RowData>,
    current_row_id: usize,
}

impl SheetReader {
    pub fn new(rows: Vec<RowData>) -> Self {
        SheetReader {
            rows,
            current_row_id: 0,
        }
    }

    pub fn get_rowid(&self) -> u32 {
        self.current_row_id as u32
    }

    pub fn move_next(&mut self) {
        self.current_row_id += 1;
    }

    pub fn has_value(&self) -> bool {
        self.rows.get(self.current_row_id).is_some()
    }

    pub fn get_value(&self, i: usize) -> Option<&CellData> {
        if let Some(row) = self.rows.get(self.current_row_id) {
            if let Some(cell_data) = row.values.as_ref() {
                return cell_data.get(i);
            }
        }

        None
    }
}
