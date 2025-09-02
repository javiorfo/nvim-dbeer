use crate::{
    dbeer::{self, Table, engine::odbc::Odbc},
    dbeer_debug,
};

pub struct Informix {
    odbc: Odbc,
}

impl Informix {
    #[allow(clippy::result_large_err)]
    pub fn connect(conn_str: &str, queries: &str) -> dbeer::Result<Self> {
        Ok(Self {
            odbc: Odbc::new(conn_str, queries)?,
        })
    }
}

impl super::SqlExecutor for Informix {
    fn select(&mut self, table: &mut Table) -> dbeer::Result {
        self.odbc.select(table)
    }

    fn execute(&mut self, table: &mut Table) -> dbeer::Result {
        self.odbc.execute(table)
    }

    fn tables(&mut self) -> dbeer::Result {
        self.odbc.queries =
            "SELECT tabname FROM systables WHERE tabtype = 'T' order by tabname;".to_string();
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
                UPPER(col.colname) AS column_name,
                MIN(
                    CASE
                        WHEN MOD(col.coltype, 256) = 0 THEN 'char'
                        WHEN MOD(col.coltype, 256) = 1 THEN 'smallint'
                        WHEN MOD(col.coltype, 256) = 2 THEN 'integer'
                        WHEN MOD(col.coltype, 256) = 3 THEN 'float'
                        WHEN MOD(col.coltype, 256) = 4 THEN 'smallfloat'
                        WHEN MOD(col.coltype, 256) = 5 THEN 'decimal'
                        WHEN MOD(col.coltype, 256) = 6 THEN 'serial'
                        WHEN MOD(col.coltype, 256) = 7 THEN 'date'
                        WHEN MOD(col.coltype, 256) = 8 THEN 'money'
                        WHEN MOD(col.coltype, 256) = 9 THEN 'null'
                        WHEN MOD(col.coltype, 256) = 10 THEN 'datetime'
                        WHEN MOD(col.coltype, 256) = 11 THEN 'byte'
                        WHEN MOD(col.coltype, 256) = 12 THEN 'text'
                        WHEN MOD(col.coltype, 256) = 13 THEN 'varchar'
                        WHEN MOD(col.coltype, 256) = 14 THEN 'interval'
                        WHEN MOD(col.coltype, 256) = 15 THEN 'nchar'
                        WHEN MOD(col.coltype, 256) = 16 THEN 'nvarchar'
                        WHEN MOD(col.coltype, 256) = 17 THEN 'int8'
                        WHEN MOD(col.coltype, 256) = 18 THEN 'serial8'
                        WHEN MOD(col.coltype, 256) = 19 THEN 'set'
                        WHEN MOD(col.coltype, 256) = 20 THEN 'multiset'
                        WHEN MOD(col.coltype, 256) = 21 THEN 'list'
                        WHEN MOD(col.coltype, 256) = 22 THEN 'row (unnamed)'
                        WHEN MOD(col.coltype, 256) = 23 THEN 'collection'
                        WHEN MOD(col.coltype, 256) = 40 THEN 'lvarchar'
                        WHEN MOD(col.coltype, 256) = 41 THEN 'boolean, blob, clob'
                        WHEN MOD(col.coltype, 256) = 43 THEN 'lvarchar (client-side only)'
                        WHEN MOD(col.coltype, 256) = 45 THEN 'boolean'
                        WHEN MOD(col.coltype, 256) = 52 THEN 'bigint'
                        WHEN MOD(col.coltype, 256) = 53 THEN 'bigserial'
                        WHEN MOD(col.coltype, 256) = 2061 THEN 'idssecuritylabel'
                        WHEN MOD(col.coltype, 256) = 4118 THEN 'row (named)'
                        ELSE 'other'
                    END
                ) AS data_type,
                MIN(CASE WHEN col.coltype >= 256 then 'YES' ELSE 'NO' END) AS not_null,
                MIN(CASE WHEN col.collength IS NULL THEN '-' ELSE col.collength::VARCHAR(20) END) AS length,
                MAX(CASE WHEN cons.constrtype = 'P' THEN 'PRIMARY KEY' WHEN cons.constrtype = 'R' THEN 'FOREIGN KEY' ELSE '-' END) AS constraint_type,
                MAX(
                    CASE
                        WHEN cons.constrtype = 'R' THEN
                            srtab.tabname || '.' || (
                                SELECT col2.colname AS referenced_column_name
                                FROM sysreferences ref2
                                JOIN sysconstraints cons2 ON ref2.primary = cons2.constrid
                                JOIN sysindexes idx2 ON cons2.idxname = idx2.idxname
                                JOIN syscolumns col2 ON idx2.tabid = col2.tabid AND idx2.part1 = col2.colno
                                WHERE ref2.ptabid = sr.ptabid AND ref2.constrid = cons.constrid
                            )
                        ELSE '-'
                    END
                ) AS referenced_table_column
                FROM systables AS tab
                JOIN syscolumns AS col ON tab.tabid = col.tabid
                LEFT JOIN sysindexes AS idx ON col.tabid = idx.tabid AND col.colno = idx.part1
                LEFT JOIN sysconstraints AS cons ON idx.idxname = cons.idxname AND tab.tabid = cons.tabid
                LEFT JOIN sysreferences AS sr ON cons.constrid = sr.constrid
                LEFT JOIN systables AS srtab ON srtab.tabid = sr.ptabid
                WHERE tab.tabname = '{}' GROUP BY col.colname, col.coltype, col.colmin, col.collength ORDER BY constraint_type desc;"#,
            self.odbc.queries
        )
    }
}
