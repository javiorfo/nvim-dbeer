use super::command::Action;
use crate::dbeer::{command::Command, engine::Postgres, error::Result, query::is_select_query, table::Table};

pub trait Runner {
    fn connect(&self) -> Result;
    fn execute(&self) -> Result;
    fn select(&mut self, table: &mut Table) -> Result;
    fn get_tables(&self) -> Result;
    fn get_table_info(&self, table: &mut Table) -> Result;
}

pub fn context(command: Command) -> Result {
    let engine: &mut dyn Runner = &mut match command.engine.as_str() {
        "postgres" => Postgres::new(&command.conn_str, &command.queries)?,
        _ => unreachable!(),
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
