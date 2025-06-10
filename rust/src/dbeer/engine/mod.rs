mod informix;
mod mongo;
mod mysql;
mod neo4j;
mod postgres;

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
    fn select(&mut self, table: &mut Table) -> dbeer::Result;
    fn execute(&mut self, table: &mut Table) -> dbeer::Result;
    fn tables(&mut self) -> dbeer::Result;
    fn table_info(&mut self, table: &mut Table) -> dbeer::Result;
    fn table_info_query(&self) -> String;
}

pub trait _NoSqlExecutor {}
