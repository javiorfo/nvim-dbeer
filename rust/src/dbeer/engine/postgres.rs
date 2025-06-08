use std::collections::HashMap;

use postgres::{
    Client, NoTls, Row,
    types::{FromSql, Type},
};

use crate::{
    dbeer::{
        self,
        dispatch::Runner,
        table::{Header, Table},
    },
    dbeer_debug,
};

pub struct Postgres<'a> {
    client: Client,
    queries: &'a str,
}

impl<'a> Postgres<'a> {
    pub fn connect(conn_str: &'a str, queries: &'a str) -> dbeer::Result<Postgres<'a>> {
        Ok(Self {
            queries,
            client: Client::connect(conn_str, NoTls).map_err(|_| {
                dbeer::Error::Msg(format!(
                    "Error connecting Postgres. Connection string: {}",
                    conn_str
                ))
            })?,
        })
    }
}

impl Runner for Postgres<'_> {
    fn select(&mut self, table: &mut Table) -> dbeer::Result {
        let results = self
            .client
            .query(self.queries, &[])
            .map_err(dbeer::Error::Postgres)?;

        let mut headers: HashMap<_, _> = results
            .first()
            .unwrap()
            .columns()
            .iter()
            .enumerate()
            .map(|(i, v)| (i + 2, Header::new(v.name())))
            .collect();

        headers.insert(1, Header::row_counter());

        let mut rows: Vec<Vec<String>> = Vec::new();
        for (i, row) in results.iter().enumerate() {
            let string_values = row_to_string(row);

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
            table.generate()?;
        }

        Ok(())
    }

    fn execute(&self) -> dbeer::Result {
        todo!()
    }

    fn get_tables(&self) -> dbeer::Result {
        Ok(())
    }

    fn get_table_info(&self, table: &mut Table) -> dbeer::Result {
        Ok(())
    }
}

fn row_to_string(row: &Row) -> Vec<String> {
    let mut result = Vec::new();

    for (i, column) in row.columns().iter().enumerate() {
        let value = match *column.type_() {
            Type::BOOL => value_or_null::<bool>(row, i),
            Type::INT2 => value_or_null::<i16>(row, i),
            Type::INT4 => value_or_null::<i32>(row, i),
            Type::INT8 => value_or_null::<i64>(row, i),
            Type::FLOAT4 => value_or_null::<f32>(row, i),
            Type::FLOAT8 => value_or_null::<f64>(row, i),
            Type::TEXT | Type::VARCHAR | Type::BPCHAR => value_or_null::<&str>(row, i),
            Type::DATE => value_or_null::<chrono::NaiveDate>(row, i),
            Type::TIMESTAMP => value_or_null::<chrono::NaiveDateTime>(row, i),
            Type::TIMESTAMPTZ => value_or_null::<chrono::DateTime<chrono::Utc>>(row, i),
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

fn value_or_null<'a, T>(row: &'a Row, i: usize) -> String
where
    T: ToString + FromSql<'a>,
{
    row.try_get::<_, Option<T>>(i)
        .ok()
        .flatten()
        .map_or(String::from("NULL"), |v| v.to_string())
}
