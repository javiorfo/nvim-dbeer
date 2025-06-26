use std::collections::HashMap;

use postgres::{
    Client, NoTls, Row,
    types::{FromSql, Type},
};

use crate::{
    dbeer::{
        self, Format, Header, Table,
        query::{is_insert_update_or_delete, split_queries},
    },
    dbeer_debug,
};

pub struct Postgres {
    client: Client,
    queries: String,
}

impl Postgres {
    #[allow(clippy::result_large_err)]
    pub fn connect(conn_str: &str, queries: &str) -> dbeer::Result<Self> {
        Ok(Self {
            queries: queries.to_string(),
            client: Client::connect(conn_str, NoTls).map_err(|_| {
                dbeer::Error::Msg(format!(
                    "Error connecting Postgres. Connection string: {}",
                    conn_str
                ))
            })?,
        })
    }

    fn row_to_string(row: &Row) -> Vec<String> {
        let mut result = Vec::new();

        for (i, column) in row.columns().iter().enumerate() {
            let value = match *column.type_() {
                Type::BOOL => Self::value_or_null::<bool>(row, i),
                Type::INT2 => Self::value_or_null::<i16>(row, i),
                Type::INT4 => Self::value_or_null::<i32>(row, i),
                Type::INT8 => Self::value_or_null::<i64>(row, i),
                Type::FLOAT4 => Self::value_or_null::<f32>(row, i),
                Type::FLOAT8 => Self::value_or_null::<f64>(row, i),
                Type::TEXT | Type::VARCHAR | Type::BPCHAR => Self::value_or_null::<&str>(row, i),
                Type::DATE => Self::value_or_null::<chrono::NaiveDate>(row, i),
                Type::TIMESTAMP => Self::value_or_null::<chrono::NaiveDateTime>(row, i),
                Type::TIMESTAMPTZ => Self::value_or_null::<chrono::DateTime<chrono::Utc>>(row, i),
                Type::INT4_ARRAY => match row.try_get::<_, Option<Vec<i32>>>(i).ok().flatten() {
                    Some(arr) => format!("{:?}", arr),
                    None => "NULL".to_string(),
                },
                ref unknown_type => {
                    dbeer_debug!("Postgres unknown type: {unknown_type}");
                    "UNKNOWN TYPE".to_string()
                }
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
}

impl super::SqlExecutor for Postgres {
    fn select(&mut self, table: &mut Table) -> dbeer::Result {
        let results = self
            .client
            .query(&self.queries, &[])
            .map_err(dbeer::Error::Postgres)?;

        if results.is_empty() {
            println!("  Query has returned 0 results.");
            return Ok(());
        }

        let mut headers: HashMap<_, _> = results
            .first()
            .unwrap()
            .columns()
            .iter()
            .enumerate()
            .map(|(i, c)| (i + 2, Header::new(c.name())))
            .collect();

        headers.insert(1, Header::row_counter());

        let mut rows: Vec<Vec<String>> = Vec::new();
        for (i, row) in results.iter().enumerate() {
            let string_values = Self::row_to_string(row);

            let mut columns = Vec::with_capacity(headers.len());
            let id_column = format!(" #{}", i + 1);
            let id_column_length = id_column.len() + 1;
            columns.push(id_column);

            let column_counter = headers.get_mut(&1).unwrap();
            if column_counter.length < id_column_length {
                column_counter.length = id_column_length;
            }

            for (column_index, value) in string_values.iter().enumerate() {
                columns.push(format!(" {}", value));
                let column = headers.get_mut(&(column_index + 2)).unwrap();
                let length = value.chars().count() + 2;
                if column.length < length {
                    column.length = length;
                }
            }
            rows.push(columns);
        }

        table.update_headers_and_rows(headers, rows)
    }

    fn execute(&mut self, table: &mut Table) -> dbeer::Result {
        let queries = split_queries(&self.queries);

        if queries.len() == 1 {
            let query = queries[0];
            let result = self
                .client
                .execute(query, &[])
                .map_err(dbeer::Error::Postgres)?;

            if is_insert_update_or_delete(query) {
                println!("  Row(s) affected: {result}");
            } else {
                println!("  Statement executed correctly.");
            }
            return Ok(());
        }

        let mut results = Vec::new();
        for (i, &query) in queries.iter().enumerate() {
            let msg = match self.client.execute(query, &[]) {
                Ok(affected) => {
                    if is_insert_update_or_delete(query) {
                        format!("{})   Row(s) affected: {}", i + 1, affected)
                    } else {
                        format!("{})   Statement executed correctly.", i + 1)
                    }
                }
                Err(e) => format!("{})   {}", i + 1, e),
            };
            results.push(msg);
        }

        table.create_execute_result_file(Format::Standard(results))
    }

    fn tables(&mut self) -> dbeer::Result {
        let table_names = self
            .client
            .query("select table_name from information_schema.tables where table_schema = 'public' order by table_name", &[])
            .map_err(dbeer::Error::Postgres)?
            .iter()
            .map(|row| {
                let name: String = row.get("table_name");
                name.to_uppercase()
            })
            .collect::<Vec<_>>().join(" ");

        dbeer_debug!("Table names: {table_names}");
        println!("[{table_names}]");

        Ok(())
    }

    fn table_info(&mut self, table: &mut Table) -> dbeer::Result {
        self.queries = self.table_info_query();
        dbeer_debug!("Table info query: {}", self.queries);
        self.select(table)
    }

    fn table_info_query(&self) -> String {
        format!(
            r#"SELECT 
                UPPER(c.column_name) AS column_name,
                c.data_type,
                CASE
                    WHEN c.is_nullable = 'YES' THEN ' '
                    ELSE ' '
                END AS not_null,
                CASE
                    WHEN c.data_type IN ('character varying', 'varchar', 'character', 'char') THEN 
                        CAST(c.character_maximum_length AS CHAR)
                    WHEN c.data_type IN ('numeric', 'decimal') THEN 
                        CAST(c.numeric_precision AS CHAR) || 
                        CASE WHEN c.numeric_scale IS NOT NULL THEN '.' || CAST(c.numeric_scale AS CHAR) ELSE '' END
                    ELSE '-'
                END AS length,
                CASE  
                    WHEN tc.constraint_type = 'PRIMARY KEY' THEN '  PRIMARY KEY'
                    WHEN tc.constraint_type = 'FOREIGN KEY' THEN '  FOREIGN KEY'
                    ELSE '-'
                END AS constraint_type,
                CASE 
                    WHEN tc.constraint_type = 'FOREIGN KEY' THEN 
                       '  ' || kcu2.table_name || '.' || kcu2.column_name
                    ELSE 
                        '-'
                END AS referenced_table_column
                FROM 
                    information_schema.columns AS c
                LEFT JOIN 
                    information_schema.key_column_usage AS kcu 
                    ON c.column_name = kcu.column_name 
                    AND c.table_name = kcu.table_name
                LEFT JOIN 
                    information_schema.table_constraints AS tc 
                    ON kcu.constraint_name = tc.constraint_name 
                    AND kcu.table_name = tc.table_name
                LEFT JOIN 
                    information_schema.referential_constraints AS rc 
                    ON tc.constraint_name = rc.constraint_name 
                    AND tc.table_schema = rc.constraint_schema
                LEFT JOIN 
                    information_schema.key_column_usage AS kcu2 
                    ON rc.unique_constraint_name = kcu2.constraint_name 
                    AND rc.unique_constraint_schema = kcu2.table_schema
                WHERE 
                    c.table_name = '{}'"#,
            self.queries
        )
    }
}
