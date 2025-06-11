use crate::{
    dbeer::{self, Table, engine::odbc::Odbc},
    dbeer_debug,
};

pub struct MsSql {
    odbc: Odbc,
}

impl MsSql {
    #[allow(clippy::result_large_err)]
    pub fn connect(conn_str: &str, queries: &str) -> dbeer::Result<Self> {
        Ok(Self {
            odbc: Odbc::new(conn_str, queries)?,
        })
    }
}

impl super::SqlExecutor for MsSql {
    fn select(&mut self, table: &mut Table) -> dbeer::Result {
        self.odbc.select(table)
    }

    fn execute(&mut self, table: &mut Table) -> dbeer::Result {
        self.odbc.execute(table)
    }

    fn tables(&mut self) -> dbeer::Result {
        self.odbc.queries = "SELECT name AS table_name FROM sys.tables order by name;".to_string();
        self.odbc.tables()
    }

    fn table_info(&mut self, table: &mut Table) -> dbeer::Result {
        self.odbc.queries = self.table_info_query();
        dbeer_debug!("Table info query: {}", self.odbc.queries);
        self.select(table)
    }

    fn table_info_query(&self) -> String {
        format!(
            r#"SELECT 
                UPPER(c.COLUMN_NAME) AS column_name,
                c.DATA_TYPE,
                CASE
                    WHEN c.IS_NULLABLE = 'YES' THEN NCHAR(0xE640) + ' '
                    ELSE NCHAR(0xF4A7) + ' '
                END AS not_null,
                CASE
                    WHEN c.CHARACTER_MAXIMUM_LENGTH IS NULL THEN '-'
                    ELSE CAST(c.CHARACTER_MAXIMUM_LENGTH AS VARCHAR)
                END AS length,
                CASE  
                    WHEN tc.CONSTRAINT_TYPE = 'PRIMARY KEY' THEN NCHAR(0xEB11) + '  PRIMARY KEY'
                    WHEN tc.CONSTRAINT_TYPE = 'FOREIGN KEY' THEN NCHAR(0xEB11) + '  FOREIGN KEY'
                    ELSE '-'
                END AS constraint_type,
                CASE 
                    WHEN tc.CONSTRAINT_TYPE = 'FOREIGN KEY' THEN 
                        NCHAR(0xEBB7) + ' ' + kcu2.TABLE_NAME + '.' + kcu2.COLUMN_NAME
                    ELSE 
                        '-'
                END AS referenced_table_column
            FROM 
                INFORMATION_SCHEMA.COLUMNS AS c
            LEFT JOIN 
                INFORMATION_SCHEMA.KEY_COLUMN_USAGE AS kcu 
                ON c.COLUMN_NAME = kcu.COLUMN_NAME 
                AND c.TABLE_NAME = kcu.TABLE_NAME
            LEFT JOIN 
                INFORMATION_SCHEMA.TABLE_CONSTRAINTS AS tc 
                ON kcu.CONSTRAINT_NAME = tc.CONSTRAINT_NAME 
                AND kcu.TABLE_NAME = tc.TABLE_NAME
            LEFT JOIN 
                INFORMATION_SCHEMA.REFERENTIAL_CONSTRAINTS AS rc 
                ON tc.CONSTRAINT_NAME = rc.CONSTRAINT_NAME 
                AND tc.TABLE_SCHEMA = rc.UNIQUE_CONSTRAINT_SCHEMA
            LEFT JOIN 
                INFORMATION_SCHEMA.KEY_COLUMN_USAGE AS kcu2 
                ON rc.UNIQUE_CONSTRAINT_NAME = kcu2.CONSTRAINT_NAME 
                AND rc.UNIQUE_CONSTRAINT_SCHEMA = kcu2.TABLE_SCHEMA
            WHERE 
                c.TABLE_NAME = '{}';"#,
            self.odbc.queries
        )
    }
}
