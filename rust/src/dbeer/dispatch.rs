use super::command::Action;
use crate::{
    dbeer::{
        self,
        command::Command,
        engine::{
            Db2, Informix, Mongo, MsSql, MySql, Oracle, Postgres, Redis, SqlExecutor, Sqlite, Type,
        },
        query::{is_select_query, strip_sql_comments},
        table::Table,
    },
    dbeer_debug,
};

#[allow(clippy::result_large_err)]
pub fn process(command: Command, engine_type: Type) -> dbeer::Result {
    match engine_type {
        Type::Sql => {
            let queries = strip_sql_comments(&command.queries);

            dbeer_debug!("Cleaned: {queries}");

            let mut engine: Box<dyn SqlExecutor> = match command.engine.as_str() {
                "postgres" => Box::new(Postgres::connect(&command.conn_str, &queries)?),
                "mysql" => Box::new(MySql::connect(
                    &command.conn_str,
                    &queries,
                    &command.db_name,
                )?),
                "informix" => Box::new(Informix::connect(&command.conn_str, &queries)?),
                "mssql" => Box::new(MsSql::connect(&command.conn_str, &queries)?),
                "oracle" => Box::new(Oracle::connect(&command.conn_str, &queries)?),
                "db2" => Box::new(Db2::connect(&command.conn_str, &queries)?),
                "sqlite" => Box::new(Sqlite::connect(&command.db_name, &queries)?),
                not_supported => {
                    return Err(dbeer::Error::Msg(format!(
                        "Engine {not_supported} is not supported"
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
        Type::Mongo => {
            let mongo = Mongo::connect(&command.conn_str, &command.db_name, &command.queries)?;

            match command.action {
                Action::Run => mongo.run(Table {
                    dest_folder: command.dest_folder,
                    ..Table::default()
                })?,
                Action::Tables => mongo.tables()?,
                Action::TableInfo => {
                    return Err(dbeer::Error::Msg(
                        "Collection info not implemented for MongoDB".to_string(),
                    ));
                }
            }
        }
        Type::Redis => {
            let mut redis = Redis::connect(&command.conn_str, &command.queries)?;

            match command.action {
                Action::Run => redis.run()?,
                _ => {
                    return Err(dbeer::Error::Msg("Not applicable for Redis".to_string()));
                }
            }
        }
        Type::Neo4j => return Err(dbeer::Error::Msg("Neo4j not implemented yet".to_string())),
    }

    Ok(())
}
