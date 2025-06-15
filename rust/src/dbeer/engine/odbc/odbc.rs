use std::collections::HashMap;

use odbc::{
    Environment,
    ResultSetState::{Data, NoData},
    Statement, Version3, create_environment_v3,
};

use crate::{
    dbeer::{
        self, Header, Table,
        engine::SqlExecutor,
        query::{is_insert_update_or_delete, split_queries},
    },
    dbeer_debug,
};

pub struct Odbc {
    pub queries: String,
    conn_str: String,
    environment: Environment<Version3>,
}

impl Odbc {
    #[allow(clippy::result_large_err)]
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
                        (i + 1) as usize,
                        Header::new(
                            &stmt
                                .describe_col(i as u16)
                                .map_err(dbeer::Error::Odbc)?
                                .name
                                .to_uppercase(),
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
                        let value = if let Some(value) = cursor
                            .get_data::<&str>(i as u16)
                            .map_err(dbeer::Error::Odbc)?
                        {
                            columns.push(format!(" {}", value));
                            value
                        } else {
                            columns.push(" NULL".to_string());
                            "NULL"
                        };

                        let column = headers.get_mut(&(i as usize + 1)).unwrap();
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

        table.update_headers_and_rows(headers, rows)
    }

    fn execute(&mut self, table: &mut Table) -> dbeer::Result {
        let queries = split_queries(&self.queries);

        if queries.len() == 1 {
            let query = queries[0];
            let connection = self
                .environment
                .connect_with_connection_string(&self.conn_str)
                .map_err(dbeer::Error::Odbc)?;

            let stmt = Statement::with_parent(&connection).map_err(dbeer::Error::Odbc)?;
            let affected_rows = match stmt
                .exec_direct(&self.queries)
                .map_err(dbeer::Error::Odbc)?
            {
                Data(stmt) => stmt.affected_row_count().map_err(dbeer::Error::Odbc)?,
                NoData(stmt) => stmt.affected_row_count().map_err(dbeer::Error::Odbc)?,
            };

            if is_insert_update_or_delete(query) {
                println!("  Row(s) affected: {}", affected_rows);
            } else {
                println!("  Statement executed correctly.");
            }
            return Ok(());
        }

        let mut results = Vec::new();
        for (i, &query) in queries.iter().enumerate() {
            let connection = self
                .environment
                .connect_with_connection_string(&self.conn_str)
                .map_err(dbeer::Error::Odbc)?;

            let stmt = Statement::with_parent(&connection).map_err(dbeer::Error::Odbc)?;

            let msg = match stmt.exec_direct(&self.queries) {
                Ok(data) => {
                    let affected_rows = match data {
                        Data(stmt) => stmt.affected_row_count().map_err(dbeer::Error::Odbc)?,
                        NoData(stmt) => stmt.affected_row_count().map_err(dbeer::Error::Odbc)?,
                    };
                    if is_insert_update_or_delete(query) {
                        format!("{})   Row(s) affected: {}", i + 1, affected_rows)
                    } else {
                        format!("{})   Statement executed correctly.", i + 1)
                    }
                }
                Err(e) => format!("{})   {}", i + 1, e),
            };

            results.push(msg);
        }

        table.create_execute_result_file(dbeer::Format::Standard(results))
    }

    fn tables(&mut self) -> crate::dbeer::Result {
        let connection = self
            .environment
            .connect_with_connection_string(&self.conn_str)
            .map_err(dbeer::Error::Odbc)?;

        let stmt = Statement::with_parent(&connection).map_err(dbeer::Error::Odbc)?;
        let mut table_names = String::new();

        if let Data(mut stmt) = stmt
            .exec_direct(&self.queries)
            .map_err(dbeer::Error::Odbc)?
        {
            let cols = stmt.num_result_cols().map_err(dbeer::Error::Odbc)?;
            while let Some(mut cursor) = stmt.fetch().map_err(dbeer::Error::Odbc)? {
                for i in 1..(cols + 1) {
                    if let Some(value) = cursor
                        .get_data::<&str>(i as u16)
                        .map_err(dbeer::Error::Odbc)?
                    {
                        table_names.push_str(&format!("{} ", value.to_uppercase()));
                    }
                }
            }
        }

        dbeer_debug!("Table names: {table_names}");
        println!("[{table_names}]");

        Ok(())
    }

    fn table_info(&mut self, _table: &mut Table) -> dbeer::Result {
        unimplemented!()
    }

    fn table_info_query(&self) -> String {
        unimplemented!()
    }
}
