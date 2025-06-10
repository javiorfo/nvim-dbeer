use std::collections::HashMap;

use mysql::{Params, Pool, PooledConn, prelude::Queryable};

use crate::{
    dbeer::{
        self, Header,
        query::{is_insert_update_or_delete, split_queries},
    },
    dbeer_debug,
};

pub struct MySql {
    queries: String,
    db_name: String,
    connection: PooledConn,
}

impl MySql {
    pub fn connect(conn_str: &str, queries: &str, db_name: &str) -> dbeer::Result<Self> {
        let pool = Pool::new(conn_str).map_err(dbeer::Error::MySql)?;

        Ok(Self {
            queries: queries.to_string(),
            db_name: db_name.to_string(),
            connection: pool.get_conn().map_err(|_| {
                dbeer::Error::Msg(format!(
                    "Error connecting MySQL. Connection string: {}",
                    conn_str
                ))
            })?,
        })
    }
}

impl super::SqlExecutor for MySql {
    fn select(&mut self, table: &mut dbeer::Table) -> dbeer::Result {
        let results = self
            .connection
            .query_iter(&self.queries)
            .map_err(dbeer::Error::MySql)?;

        let results_columns = results.columns().as_ref().to_vec();
        let mut headers: HashMap<_, _> = results_columns
            .iter()
            .enumerate()
            .map(|(i, c)| (i + 2, Header::new(c.name_str().as_ref())))
            .collect();

        headers.insert(1, Header::row_counter());

        let mut rows: Vec<Vec<String>> = Vec::new();
        for (i, row) in results.enumerate() {
            let row_value = row.map_err(dbeer::Error::MySql)?;

            let mut columns = Vec::with_capacity(headers.len());
            let id_column = format!(" #{}", i + 1);
            let id_column_length = id_column.len() + 1;
            columns.push(id_column);

            let column_counter = headers.get_mut(&1).unwrap();
            if column_counter.length < id_column_length {
                column_counter.length = id_column_length;
            }

            for column_index in 0..results_columns.len() {
                let value: String = row_value.get(column_index).unwrap_or("NULL".to_string());
                columns.push(format!(" {}", value));
                let column = headers.get_mut(&(column_index + 2)).unwrap();
                let length = value.chars().count() + 2;
                if column.length < length {
                    column.length = length;
                }
            }
            rows.push(columns);
        }

        if rows.is_empty() {
            println!("  Query has returned 0 results.");
            return Ok(());
        }

        table.headers = headers;
        table.rows = rows;

        dbeer_debug!("Generating dbeer table...");
        table.generate()?;

        Ok(())
    }

    fn execute(&mut self, table: &mut dbeer::Table) -> dbeer::Result {
        let queries = split_queries(&self.queries);

        if queries.len() == 1 {
            let query = queries[0];
            self.connection
                .exec_drop(query, Params::Empty)
                .map_err(dbeer::Error::MySql)?;

            if is_insert_update_or_delete(query) {
                println!("  Row(s) affected: {}", self.connection.affected_rows());
            } else {
                println!("  Statement executed correctly.");
            }
            return Ok(());
        }

        let mut results = Vec::new();
        for (i, &query) in queries.iter().enumerate() {
            let msg = match self.connection.exec_drop(query, Params::Empty) {
                Ok(_) => {
                    if is_insert_update_or_delete(query) {
                        format!(
                            "{})   Row(s) affected: {}",
                            i + 1,
                            self.connection.affected_rows()
                        )
                    } else {
                        format!("{})   Statement executed correctly.", i + 1)
                    }
                }
                Err(e) => format!("{})   {}", i + 1, e),
            };
            results.push(msg);
        }

        let filepath = table.create_dbeer_file_format();
        println!("syn match dbeerStmtErr ' ' | hi link dbeerStmtErr ErrorMsg");
        println!("{filepath}");
        table.write_to_file(&filepath, &results)?;

        Ok(())
    }

    fn tables(&mut self) -> dbeer::Result {
        let results = self
            .connection
            .query_iter(format!("select table_name from information_schema.tables where table_schema = '{}' order by table_name",
                self.db_name
            ))
            .map_err(dbeer::Error::MySql)?;

        let mut table_names = Vec::new();

        for row in results {
            let row = row.map_err(dbeer::Error::MySql)?;
            let table_name: String = row.get(0).unwrap_or_default();
            table_names.push(table_name.to_uppercase());
        }

        let formatted = table_names.join(" ");
        dbeer_debug!("Table names: {formatted}");
        println!("[{formatted}]");

        Ok(())
    }

    fn table_info(&mut self, table: &mut dbeer::Table) -> dbeer::Result {
        self.queries = self.table_info_query();
        dbeer_debug!("Table info query: {}", self.queries);
        self.select(table)?;
        Ok(())
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
                    WHEN c.character_maximum_length IS NULL THEN '-'
                    ELSE CAST(c.character_maximum_length AS CHAR)
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
