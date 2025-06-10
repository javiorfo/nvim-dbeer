mod informix;
mod mongo;
mod mysql;
mod neo4j;
pub mod odbc;
mod postgres;

pub use informix::Informix;
pub use mysql::MySql;
pub use postgres::Postgres;

use crate::dbeer::{self, table::Table};

pub enum Type {
    Sql,
    Mongo,
    Neo4j,
    Redis,
}

impl From<String> for Type {
    fn from(value: String) -> Self {
        match value.as_str() {
            "mongo" => Type::Mongo,
            "neo4j" => Type::Neo4j,
            "redis" => Type::Redis,
            _ => Type::Sql,
        }
    }
}

pub trait SqlExecutor {
    #[allow(clippy::result_large_err)]
    fn select(&mut self, table: &mut Table) -> dbeer::Result;

    #[allow(clippy::result_large_err)]
    fn execute(&mut self, table: &mut Table) -> dbeer::Result;

    #[allow(clippy::result_large_err)]
    fn tables(&mut self) -> dbeer::Result;

    #[allow(clippy::result_large_err)]
    fn table_info(&mut self, table: &mut Table) -> dbeer::Result;

    fn table_info_query(&self) -> String;
}

pub trait _NoSqlExecutor {}
