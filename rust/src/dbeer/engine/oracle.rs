use crate::{
    dbeer::{self, Table, engine::odbc::Odbc},
    dbeer_debug,
};

pub struct Oracle {
    odbc: Odbc,
}

impl Oracle {
    #[allow(clippy::result_large_err)]
    pub fn connect(conn_str: &str, queries: &str) -> dbeer::Result<Self> {
        Ok(Self {
            odbc: Odbc::new(conn_str, queries)?,
        })
    }
}

impl super::SqlExecutor for Oracle {
    fn select(&mut self, table: &mut Table) -> dbeer::Result {
        self.odbc.select(table)
    }

    fn execute(&mut self, table: &mut Table) -> dbeer::Result {
        self.odbc.execute(table)
    }

    fn tables(&mut self) -> dbeer::Result {
        self.odbc.queries =
            "select table_name from all_tables where owner = 'PUBLIC' order by table_name;"
                .to_string();
        self.odbc.tables()
    }

    fn table_info(&mut self, table: &mut Table) -> dbeer::Result {
        self.odbc.queries = self.table_info_query();
        dbeer_debug!("Table info query: {}", self.odbc.queries);
        self.select(table)?;
        Ok(())
    }

    fn table_info_query(&self) -> String {
        format!(
            r#"SELECT 
                UPPER(c.column_name) AS column_name,
                c.data_type,
                CASE
                    WHEN c.nullable = 'Y' THEN ' '
                    ELSE ' '
                END AS not_null,
                CASE
                    WHEN c.data_type IN ('VARCHAR2', 'CHAR', 'NCHAR', 'NVARCHAR2') THEN 
                        COALESCE(c.data_length, '-') 
                    ELSE 
                        '-'
                END AS length,
                CASE  
                    WHEN con.constraint_type = 'P' THEN '  PRIMARY KEY'
                    WHEN con.constraint_type = 'R' THEN '  FOREIGN KEY'
                    ELSE '-'
                END AS constraint_type,
                CASE 
                    WHEN con.constraint_type = 'R' THEN 
                        '  ' || rcc.table_name || '.' || rcc.column_name
                    ELSE 
                        '-'
                END AS referenced_table_column
                FROM user_tab_columns c
                LEFT JOIN user_cons_columns kcu ON c.column_name = kcu.column_name 
                    AND c.table_name = kcu.table_name
                LEFT JOIN user_constraints con ON kcu.constraint_name = con.constraint_name 
                    AND kcu.table_name = con.table_name
                LEFT JOIN user_constraints rc ON con.constraint_name = rc.constraint_name 
                    AND con.table_name = rc.table_name AND con.constraint_type = 'R'
                LEFT JOIN user_cons_columns rcc ON rc.r_constraint_name = rcc.constraint_name
                WHERE c.table_name = '{}';"#,
            self.odbc.queries
        )
    }
}
