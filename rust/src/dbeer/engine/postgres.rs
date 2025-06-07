use std::collections::HashMap;

use postgres::{
    Client, NoTls, Row,
    types::{FromSql, Type},
};

use crate::{
    dbeer::{
        dispatch::Runner,
        error::{DBeerError, Result},
        table::{Header, Table},
    },
    dbeer_debug,
};

pub struct Postgres<'a> {
    client: Client,
    queries: &'a str,
}

impl<'a> Postgres<'a> {
    pub fn new(conn_str: &'a str, queries: &'a str) -> Result<Postgres<'a>> {
        Ok(Self {
            queries,
            client: Client::connect(conn_str, NoTls).map_err(|_| {
                DBeerError::Custom(format!("Error connecting Postgres CONN STR: {}", conn_str))
            })?,
        })
    }
}

impl Postgres<'_> {
    fn row_to_strings(row: &Row) -> Vec<String> {
        let mut result = Vec::new();

        for (i, column) in row.columns().iter().enumerate() {
            let value = match *column.type_() {
                Type::BOOL => Self::get_value_or_null_string::<bool>(row, i),
                Type::INT2 => Self::get_value_or_null_string::<i16>(row, i),
                Type::INT4 => Self::get_value_or_null_string::<i32>(row, i),
                Type::INT8 => Self::get_value_or_null_string::<i64>(row, i),
                Type::FLOAT4 => Self::get_value_or_null_string::<f32>(row, i),
                Type::FLOAT8 => Self::get_value_or_null_string::<f64>(row, i),
                Type::TEXT | Type::VARCHAR | Type::BPCHAR => {
                    Self::get_value_or_null_string::<&str>(row, i)
                }
                Type::DATE => Self::get_value_or_null_string::<chrono::NaiveDate>(row, i),
                Type::TIMESTAMP => Self::get_value_or_null_string::<chrono::NaiveDateTime>(row, i),
                Type::TIMESTAMPTZ => {
                    Self::get_value_or_null_string::<chrono::DateTime<chrono::Utc>>(row, i)
                }
                Type::INT4_ARRAY => match row.try_get::<_, Option<Vec<i32>>>(i).ok().flatten() {
                    Some(arr) => format!("{:?}", arr),
                    None => "NULL".to_string(),
                },
                _ => "UNKNOWN TYPE".to_string(),
            };
            result.push(value);
        }

        result
    }

    fn get_value_or_null_string<'a, T>(row: &'a Row, i: usize) -> String
    where
        T: ToString + FromSql<'a>,
    {
        row.try_get::<_, Option<T>>(i)
            .ok()
            .flatten()
            .map_or(String::from("NULL"), |v| v.to_string())
    }
}

impl Runner for Postgres<'_> {
    fn connect(&self) -> Result {
        todo!()
    }

    fn execute(&self) -> Result {
        todo!()
    }

    fn select(&mut self, table: &mut Table) -> Result {
        let results = self.client.query(self.queries, &[]).unwrap();

        let mut headers: HashMap<_, _> = results
            .first()
            .unwrap()
            .columns()
            .iter()
            .enumerate()
            .map(|(i, v)| {
                (
                    i + 2,
                    Header {
                        name: format!(" {}", v.name().to_uppercase()),
                        length: v.name().len() + 2,
                    },
                )
            })
            .collect();

        headers.insert(
            1,
            Header {
                name: "  ".to_string(),
                length: 4,
            },
        );

        let mut rows: Vec<Vec<String>> = Vec::new();
        for (i, row) in results.iter().enumerate() {
            let string_values = Self::row_to_strings(row);

            let mut columns = Vec::with_capacity(headers.len());
            columns.push(format!(" #{}", i + 1));

            for (column_index, value) in string_values.iter().enumerate() {
                columns.push(format!(" {}", value));
                let column = headers.get_mut(&(column_index + 2)).unwrap();
                let length = value.len() + 2;
                if column.length < length {
                    column.length = length;
                }
            }
            rows.push(columns);
        }

        table.headers = headers;
        table.rows = rows;

        if table.rows.is_empty() {
            println!("  Query has returned 0 results.");
        } else {
            dbeer_debug!("Generating dbeer table...");
            table.generate();
        }

        Ok(())
    }

    fn get_tables(&self) -> Result {
        Ok(())
    }

    fn get_table_info(&self, table: &mut Table) -> Result {
        Ok(())
    }
}
