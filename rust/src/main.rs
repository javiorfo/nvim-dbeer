use std::env;

use crate::dbeer::{
    command::Command,
    dispatch::process,
    logger::{debug, error, logger_init},
};
mod dbeer;

fn main() {
    let mut dbeer_log_file = String::new();
    let mut log_debug = false;
    let mut command = Command::new();

    let args: Vec<String> = env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-engine" => command.engine = args[i + 1].clone(),
            "-conn-str" => command.conn_str = args[i + 1].clone(),
            "-dbname" => command.db_name = args[i + 1].clone(),
            "-queries" => command.queries = args[i + 1].clone(),
            "-border-style" => command.border_style = args[i + 1].clone().into(),
            "-dest-folder" => command.dest_folder = args[i + 1].clone(),
            "-dbeer-log-file" => dbeer_log_file = args[i + 1].clone(),
            "-option" => command.action = args[i + 1].clone().into(), // TODO action in Lua
            "-header-style-link" => command.header_style_link = args[i + 1].clone(),
            "-log-debug" => log_debug = args[i + 1].clone().parse().unwrap_or(log_debug),
            _ => break,
        }
        i += 2;
    }

    logger_init(&dbeer_log_file, log_debug).expect("Logger init failed!");

    dbeer_debug!("Debug enabled!");
    dbeer_debug!("Parsed params: {command:#?}");

    let engine_type = command.engine.clone();
    if let Err(e) = process(command, engine_type.into()) {
        let error_msg = format!("[ERROR] {}", e);
        println!("{error_msg}");
        dbeer_error!("{error_msg}");
    }
}
