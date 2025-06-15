use crate::dbeer::{self, Table, engine::odbc::Odbc};

pub struct Db2 {
    odbc: Odbc,
}

impl Db2 {
    #[allow(clippy::result_large_err)]
    pub fn connect(conn_str: &str, queries: &str) -> dbeer::Result<Self> {
        Ok(Self {
            odbc: Odbc::new(conn_str, queries)?,
        })
    }
}

impl super::SqlExecutor for Db2 {
    fn select(&mut self, table: &mut Table) -> dbeer::Result {
        self.odbc.select(table)
    }

    fn execute(&mut self, table: &mut Table) -> dbeer::Result {
        self.odbc.execute(table)
    }

    fn tables(&mut self) -> dbeer::Result {
        self.odbc.queries =
            "select tabname as table_name from syscat.tables where tabschema = 'PUBLIC' order by tabname;"
                .to_string();
        self.odbc.tables()
    }

    fn table_info(&mut self, _table: &mut Table) -> dbeer::Result {
        Err(dbeer::Error::Msg(
            "Table info not implemented in DB2".to_string(),
        ))
    }

    fn table_info_query(&self) -> String {
        unimplemented!()
    }
}
