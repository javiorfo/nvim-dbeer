use super::command::Action;
use crate::dbeer::{
    self,
    command::Command,
    engine::{Postgres, SqlExecutor, Type},
    query::{is_select_query, strip_sql_comments},
    table::Table,
};

pub fn process(command: Command, engine_type: Type) -> dbeer::Result {
    match engine_type {
        Type::Sql => {
            let queries = strip_sql_comments(&command.queries);

            let engine: &mut dyn SqlExecutor = &mut match command.engine.as_str() {
                "postgres" => Postgres::connect(&command.conn_str, &queries)?,
                not_supported => {
                    return Err(dbeer::Error::Msg(format!(
                        "Engine {} is not supported",
                        not_supported
                    )));
                }
            };

            match command.action {
                Action::Run => {
                    let table = &mut Table::new(
                        command.dest_folder,
                        command.header_style_link,
                        command.border_style,
                    );

                    if is_select_query(&queries) {
                        engine.select(table)?;
                    } else {
                        engine.execute(table)?;
                    }
                }
                Action::Tables => engine.tables()?,
                Action::TableInfo => engine.table_info(&mut Table::new(
                    command.dest_folder,
                    command.header_style_link,
                    command.border_style,
                ))?,
            }
        }
        Type::Mongo => return Err(dbeer::Error::Msg("Mongo not implemented yet".to_string())),
        Type::Neo4j => return Err(dbeer::Error::Msg("Neo4j not implemented yet".to_string())),
        Type::Redis => return Err(dbeer::Error::Msg("Redis not implemented yet".to_string())),
    }

    Ok(())
}
