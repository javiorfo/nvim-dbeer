use chrono::Local;

use super::border::BorderStyle;
use crate::{dbeer, dbeer_debug};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};

#[derive(Debug)]
pub struct Header {
    pub name: String,
    pub length: usize,
}

impl Header {
    pub fn row_counter() -> Self {
        Self {
            name: "  ".to_string(),
            length: 4,
        }
    }

    pub fn new(name: &str) -> Self {
        Header {
            name: format!(" {}", name.to_uppercase()),
            length: name.len() + 2,
        }
    }
}

#[derive(Debug, Default)]
pub struct Table {
    pub dest_folder: String,
    pub header_style_link: String,
    pub border_style: BorderStyle,
    pub headers: HashMap<usize, Header>,
    pub rows: Vec<Vec<String>>,
}

impl Table {
    const DBEER_EXTENSION: &str = "dbeer";

    pub fn new(dest_folder: String, header_style_link: String, border_style: BorderStyle) -> Self {
        Self {
            dest_folder,
            header_style_link,
            border_style,
            ..Table::default()
        }
    }

    #[allow(clippy::result_large_err)]
    pub fn update_headers_and_rows(
        &mut self,
        headers: HashMap<usize, Header>,
        rows: Vec<Vec<String>>,
    ) -> dbeer::Result {
        self.headers = headers;
        self.rows = rows;

        dbeer_debug!("Generating dbeer table...");
        self.generate()
    }

    #[allow(clippy::result_large_err)]
    pub fn create_execute_result_file(&self, results: Vec<String>) -> dbeer::Result {
        let filepath = self.create_dbeer_file_format();
        println!("syn match dbeerStmtErr ' ' | hi link dbeerStmtErr ErrorMsg");
        println!("{filepath}");

        dbeer_debug!("File path: {filepath}. Results {results:#?}");

        self.write_to_file(&filepath, &results)
    }

    #[allow(clippy::result_large_err)]
    pub fn generate(&self) -> dbeer::Result {
        let border = self.border_style.get();

        // header
        let mut header_up = String::from(border.corner_up_left);
        let mut header_mid = String::from(border.vertical);
        let mut header_bottom = String::from(border.vertical_left);

        let headers_len = self.headers.len();
        for key in 1..headers_len + 1 {
            let length = self.headers.get(&key).unwrap().length;
            header_up.push_str(&border.horizontal.repeat(length));
            header_bottom.push_str(&border.horizontal.repeat(length));
            header_mid.push_str(&Self::add_spaces(
                &self.headers.get(&key).unwrap().name,
                length,
            ));
            header_mid.push_str(border.vertical);

            if key < headers_len {
                header_up.push_str(border.division_up);
                header_bottom.push_str(border.intersection);
            } else {
                header_up.push_str(border.corner_up_right);
                header_bottom.push_str(border.vertical_right);
            }
        }

        // results
        let mut table: Vec<String> = vec![header_up, header_mid, header_bottom];
        let rows_len = self.rows.len() - 1;
        let row_fields_len = self.rows.first().unwrap().len() - 1;

        for (i, row) in self.rows.iter().enumerate() {
            let mut value = String::from(border.vertical);
            let mut line = String::new();

            line.push_str(if i < rows_len {
                border.vertical_left
            } else {
                border.corner_bottom_left
            });

            for (j, field) in row.iter().enumerate() {
                let length = self.headers.get(&(j + 1)).unwrap().length;

                value.push_str(&Self::add_spaces(field, length));
                value.push_str(border.vertical);

                line.push_str(&border.horizontal.repeat(length));

                line.push_str(match (i < rows_len, j < row_fields_len) {
                    (true, true) => border.intersection,
                    (true, false) => border.vertical_right,
                    (false, true) => border.division_bottom,
                    (false, false) => border.corner_bottom_right,
                });
            }
            table.push(value);
            table.push(line);
        }

        let filepath = self.create_dbeer_file_format();

        dbeer_debug!("File path: {}", filepath);
        dbeer_debug!("Table: {:#?}", table);

        println!("{}", Self::hi(&self.headers, &self.header_style_link));
        println!("{filepath}");

        self.write_to_file(&filepath, &table)
    }

    fn add_spaces(input_string: &str, len: usize) -> String {
        let mut result = String::from(input_string);
        let input_str_len = input_string.chars().count();

        if len > input_str_len {
            let diff = len - input_str_len;
            result.push_str(&" ".repeat(diff));
        }

        result
    }

    #[allow(clippy::result_large_err)]
    pub fn write_to_file(&self, filepath: &str, strings: &[String]) -> dbeer::Result {
        let file = File::create(filepath).map_err(dbeer::Error::Io)?;
        let mut writer = BufWriter::new(file);

        for v in strings {
            let line = format!("{}\n", v);
            writer
                .write_all(line.as_bytes())
                .map_err(dbeer::Error::Io)?;
        }

        writer.flush().map_err(dbeer::Error::Io)
    }

    fn hi(headers: &HashMap<usize, Header>, style: &str) -> String {
        let matches = headers
            .iter()
            .map(|(k, v)| {
                format!(
                    "syn match header{} '\\<{}\\>' | hi link header{} {} |",
                    k,
                    v.name.trim(),
                    k,
                    style
                )
            })
            .collect::<Vec<_>>()
            .join(" ");

        dbeer_debug!("Highlight matches: {}", matches);
        matches
    }

    pub fn create_dbeer_file_format(&self) -> String {
        let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
        format!(
            "{}/{}.{}",
            self.dest_folder,
            timestamp,
            Self::DBEER_EXTENSION
        )
    }

    fn _create_dbeer_mongo_file_format(dest_folder: &str) -> String {
        let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
        format!(
            "{}/{}.{}.{}",
            dest_folder,
            timestamp,
            Self::DBEER_EXTENSION,
            "json"
        )
    }
}
