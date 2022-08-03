use crate::{
    error::{SheetError, SheetError::Api},
    range::Range,
    reader::SheetReader,
};
use google_sheets_api::{client::GoogleSheetsReadOnlyClient, RowData};
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct Sheet {
    #[builder]
    client: GoogleSheetsReadOnlyClient,
    #[builder(default)]
    rows: Vec<RowData>,
    #[builder(setter(into))]
    id: String,
    #[builder(setter(into))]
    sheet: String,
    #[builder(setter(into))]
    range: Range,
}

impl Sheet {
    pub fn open(&mut self) -> Result<(), SheetError> {
        match self.client.get(&self.id, &self.sheet, &self.range) {
            Ok(sheet) => {
                self.rows = sheet
                    .sheets
                    .unwrap()
                    .get(0)
                    .unwrap() // there only is a sheet
                    .data
                    .as_ref()
                    .unwrap() // there should be a range, hence should have a row data
                    .get(0)
                    .unwrap()
                    .row_data
                    .as_ref()
                    .unwrap()
                    .to_vec();
                Ok(())
            }
            Err(why) => Err(Api(why)),
        }
    }

    pub fn get_reader(&mut self) -> SheetReader {
        SheetReader::new(self.rows.clone())
    }

    pub fn get_columns(&mut self) -> Vec<String> {
        if let Some(row) = self.rows.get(0) {
            if let Some(cell_data) = row.values.as_ref() {
                if !cell_data.is_empty() {
                    return (0..cell_data.len())
                        .into_iter()
                        .map(|n| number_to_column_name(n + column_name_to_number(&self.range.c1)))
                        .collect();
                }
            }
        }

        Vec::new()
    }
}

fn column_name_to_number(name: impl Into<String>) -> usize {
    let mut num = 0;

    for c in name.into().chars() {
        num = (c as usize) - 64 + num * 26;
    }

    num
}

fn number_to_column_name(num: usize) -> String {
    let mut num = num;
    let mut column_name = String::from("");

    while num > 0 {
        let modulo = (num - 1) % 26;
        num = (num - modulo) / 26;
        column_name = format!(
            "{}{}",
            std::char::from_u32((65 + modulo).try_into().unwrap()).unwrap(),
            column_name
        );
    }

    column_name.trim().to_string()
}

#[cfg(test)]
mod tests {
    use crate::sheet::{column_name_to_number, number_to_column_name};

    #[test]
    fn test_number_to_column_name() {
        assert_eq!("A", number_to_column_name(1));
        assert_eq!("Z", number_to_column_name(26));
        assert_eq!("AA", number_to_column_name(27));
        assert_eq!("AZ", number_to_column_name(52));
        assert_eq!("JJ", number_to_column_name(270));
        assert_eq!("VDX", number_to_column_name(15000));
    }

    #[test]
    fn test_column_name_to_number() {
        assert_eq!(1, column_name_to_number("A"));
        assert_eq!(26, column_name_to_number("Z"));
        assert_eq!(27, column_name_to_number("AA"));
        assert_eq!(52, column_name_to_number("AZ"));
        assert_eq!(270, column_name_to_number("JJ"));
        assert_eq!(15000, column_name_to_number("VDX"));
    }
}
