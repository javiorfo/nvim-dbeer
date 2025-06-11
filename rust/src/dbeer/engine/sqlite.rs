use std::collections::HashMap;

use sqlite::{Connection, State};

use crate::{
    dbeer::{
        self, Header, Table,
        query::{is_insert_update_or_delete, split_queries},
    },
    dbeer_debug,
};

pub struct Sqlite {
    queries: String,
    connection: Connection,
}

impl Sqlite {
    #[allow(clippy::result_large_err)]
    pub fn connect(db_name: &str, queries: &str) -> dbeer::Result<Self> {
        Ok(Self {
            queries: queries.to_string(),
            connection: sqlite::open(db_name).map_err(dbeer::Error::Sqlite)?,
        })
    }
}

impl super::SqlExecutor for Sqlite {
    fn select(&mut self, table: &mut Table) -> dbeer::Result {
        let mut headers = HashMap::new();
        headers.insert(1, Header::row_counter());

        let mut stmt = self
            .connection
            .prepare(&self.queries)
            .map_err(dbeer::Error::Sqlite)?;

        for (i, name) in stmt.column_names().iter().enumerate() {
            headers.insert(i + 2, Header::new(name));
        }

        let mut rows = Vec::new();
        let mut counter = 1;
        while let Ok(State::Row) = stmt.next() {
            let mut columns = Vec::with_capacity(headers.len());
            let id_column = format!(" #{}", counter);
            let id_column_length = id_column.len() + 1;
            columns.push(id_column);

            let column_counter = headers.get_mut(&1).unwrap();
            if column_counter.length < id_column_length {
                column_counter.length = id_column_length;
            }
            counter += 1;

            for i in 0..stmt.column_count() {
                let value: String = stmt.read(i).map_err(dbeer::Error::Sqlite)?;
                columns.push(format!(" {}", value));
                let column = headers.get_mut(&(i + 2)).unwrap();
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

    fn execute(&mut self, table: &mut Table) -> dbeer::Result {
        let queries = split_queries(&self.queries);

        if queries.len() == 1 {
            let query = queries[0];
            self.connection
                .execute(query)
                .map_err(dbeer::Error::Sqlite)?;

            if is_insert_update_or_delete(query) {
                println!("  Row(s) affected: {}", self.connection.change_count());
            } else {
                println!("  Statement executed correctly.");
            }
            return Ok(());
        }

        let mut results = Vec::new();
        for (i, &query) in queries.iter().enumerate() {
            let msg = match self.connection.execute(query) {
                Ok(_) => {
                    if is_insert_update_or_delete(query) {
                        format!(
                            "{})   Row(s) affected: {}",
                            i + 1,
                            self.connection.change_count()
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

    fn tables(&mut self) -> crate::dbeer::Result {
        let mut table_names = String::new();

        self.connection
            .iterate(
                "select name from sqlite_master where type = 'table' order by name;",
                |pairs| {
                    for (_, value) in pairs.iter() {
                        table_names.push_str(&format!("{} ", value.unwrap_or("-").to_uppercase()));
                    }
                    true
                },
            )
            .map_err(dbeer::Error::Sqlite)?;

        dbeer_debug!("Table names: {table_names}");
        println!("[{table_names}]");

        Ok(())
    }

    fn table_info(&mut self, table: &mut Table) -> dbeer::Result {
        self.queries = self.table_info_query();
        dbeer_debug!("Table info query: {}", self.queries);
        self.select(table)?;
        Ok(())
    }

    fn table_info_query(&self) -> String {
        format!(r#"PRAGMA table_info("{}")"#, self.queries)
    }
}
