use super::command::Action;
use crate::dbeer::{
    self, command::Command, engine::Postgres, query::is_select_query, table::Table,
};

pub trait Runner {
    fn select(&mut self, table: &mut Table) -> dbeer::Result;
    fn execute(&self) -> dbeer::Result;
    fn get_tables(&self) -> dbeer::Result;
    fn get_table_info(&self, table: &mut Table) -> dbeer::Result;
}

pub fn context(command: Command) -> dbeer::Result {
    let engine: &mut dyn Runner = &mut match command.engine.as_str() {
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

    Ok(())
}
