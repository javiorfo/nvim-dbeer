use crate::dbeer::BorderStyle;

#[derive(Debug, Default)]
pub struct Command {
    pub engine: String,
    pub conn_str: String,
    pub db_name: String,
    pub queries: String,
    pub border_style: BorderStyle,
    pub dest_folder: String,
    pub header_style_link: String,
    pub action: Action,
}

impl Command {
    pub fn new() -> Self {
        Self {
            dest_folder: "/tmp".to_string(),
            header_style_link: "Type".to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default)]
pub enum Action {
    #[default]
    Run,
    Tables,
    TableInfo,
}

impl From<String> for Action {
    fn from(value: String) -> Self {
        match value.as_str() {
            "1" => Action::Run,
            "2" => Action::Tables,
            "3" => Action::TableInfo,
            _ => unreachable!(),
        }
    }
}
