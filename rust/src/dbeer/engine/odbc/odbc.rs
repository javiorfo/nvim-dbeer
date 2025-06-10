use std::collections::HashMap;

use odbc::{
    Environment,
    ResultSetState::{Data, NoData},
    Statement, Version3, create_environment_v3,
};

use crate::{
    dbeer::{self, Header, Table, engine::SqlExecutor},
    dbeer_debug,
};

pub struct Odbc {
    pub queries: String,
    conn_str: String,
    environment: Environment<Version3>,
}

impl Odbc {
    pub fn new(conn_str: &str, queries: &str) -> dbeer::Result<Self> {
        let environment = create_environment_v3().map_err(|e| dbeer::Error::Odbc(e.unwrap()))?;
        Ok(Self {
            queries: queries.to_string(),
            conn_str: conn_str.to_string(),
            environment,
        })
    }
}

impl SqlExecutor for Odbc {
    fn select(&mut self, table: &mut Table) -> dbeer::Result {
        let connection = self
            .environment
            .connect_with_connection_string(&self.conn_str)
            .map_err(dbeer::Error::Odbc)?;

        let stmt = Statement::with_parent(&connection).map_err(dbeer::Error::Odbc)?;
        let mut headers = HashMap::new();
        headers.insert(1, Header::row_counter());
        let mut rows: Vec<Vec<String>> = Vec::new();

        match stmt
            .exec_direct(&self.queries)
            .map_err(dbeer::Error::Odbc)?
        {
            Data(mut stmt) => {
                let columns_len = stmt.num_result_cols().map_err(dbeer::Error::Odbc)?;

                for i in 1..=columns_len {
                    headers.insert(
                        (i + 2) as usize,
                        Header::new(
                            &stmt
                                .describe_col(i as u16)
                                .map_err(dbeer::Error::Odbc)?
                                .name,
                        ),
                    );
                }

                let mut col_counter = 1;

                while let Some(mut cursor) = stmt.fetch().map_err(dbeer::Error::Odbc)? {
                    let mut columns = Vec::with_capacity(headers.len());
                    let id_column = format!(" #{}", col_counter);
                    let id_column_length = id_column.len() + 1;
                    columns.push(id_column);

                    let column_counter = headers.get_mut(&1).unwrap();
                    if column_counter.length < id_column_length {
                        column_counter.length = id_column_length;
                    }
                    col_counter += 1;

                    for i in 1..=columns_len {
                        let value = match cursor
                            .get_data::<String>(i as u16)
                            .map_err(dbeer::Error::Odbc)?
                        {
                            Some(value) => {
                                columns.push(format!(" {}", value));
                                value
                            }
                            None => {
                                columns.push(" NULL".to_string());
                                "NULL".to_string()
                            }
                        };

                        let column = headers.get_mut(&(i as usize + 2)).unwrap();
                        let length = value.chars().count() + 2;
                        if column.length < length {
                            column.length = length;
                        }
                    }
                    rows.push(columns);
                }
            }
            NoData(_) => {
                println!("  Query has returned 0 results.");
                return Ok(());
            }
        }

        table.headers = headers;
        table.rows = rows;

        dbeer_debug!("Generating dbeer table...");
        table.generate()?;

        Ok(())
    }

    fn execute(&mut self, table: &mut Table) -> dbeer::Result {
        todo!()
    }

    fn tables(&mut self) -> crate::dbeer::Result {
        todo!()
    }

    fn table_info(&mut self, _table: &mut Table) -> dbeer::Result {
        unimplemented!()
    }

    fn table_info_query(&self) -> String {
        unimplemented!()
    }
}
