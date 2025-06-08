use super::command::Action;
use crate::dbeer::{
    self, command::Command, engine::Postgres, query::is_select_query, table::Table,
};

pub trait SqlExecutor {
    fn select(&mut self, table: &mut Table) -> dbeer::Result;
    fn execute(&self) -> dbeer::Result;
    fn get_tables(&self) -> dbeer::Result;
    fn get_table_info(&self, table_name: &str) -> String;
}

pub trait NoSqlExecutor {}

pub fn context(command: Command, engine_type: EngineType) -> dbeer::Result {
    match engine_type {
        EngineType::Sql => {
            let engine: &mut dyn SqlExecutor = &mut match command.engine.as_str() {
                "postgres" => Postgres::connect(&command.conn_str, &command.queries)?,
                _ => return Err(dbeer::Error::Msg("".to_string())),
            };

            match command.action {
                Action::Run => {
                    if is_select_query(&command.queries) {
                        engine.select(&mut Table::new(
                            command.dest_folder,
                            command.header_style_link,
                            command.border_style,
                        ))?;
                    } else {
                        engine.execute()?;
                    }
                }
                Action::Tables => todo!(),
                Action::TableInfo => todo!(),
            }
        }
        EngineType::NoSql => todo!(),
        EngineType::Graph => todo!(),
        EngineType::KeyValue => todo!(),
    }

    Ok(())
}

pub enum EngineType {
    Sql,
    NoSql,
    Graph,
    KeyValue,
}

impl EngineType {
    pub fn new(engine: &str) -> EngineType {
        match engine {
            "mongo" => EngineType::NoSql,
            "neo4j" => EngineType::Graph,
            "redis" => EngineType::KeyValue,
            _ => EngineType::Sql,
        }
    }
}
