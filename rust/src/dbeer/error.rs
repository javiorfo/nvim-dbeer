#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Error {
    Io(std::io::Error),
    Postgres(postgres::Error),
    Mongo(mongodb::error::Error),
    Bson(mongodb::bson::ser::Error),
    MySql(mysql::Error),
    Odbc(odbc::DiagnosticRecord),
    Sqlite(sqlite::Error),
    Serde(serde_json::Error),
    Msg(String),
}

pub type Result<T = ()> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "IO error => {e}"),
            Error::Postgres(e) => write!(f, "Postgres error => {e}"),
            Error::Mongo(e) => write!(f, "Mongo error => {e}"),
            Error::Bson(e) => write!(f, "Bson error => {e}"),
            Error::MySql(e) => write!(f, "MySql error => {e}"),
            Error::Sqlite(e) => write!(f, "Sqlite error => {e}"),
            Error::Odbc(e) => write!(
                f,
                "Odbc error => {}",
                std::str::from_utf8(e.get_raw_message()).unwrap_or("No info available")
            ),
            Error::Serde(e) => write!(f, "JSON parser error => {e}"),
            Error::Msg(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::Postgres(e) => Some(e),
            Error::Mongo(e) => Some(e),
            Error::Bson(e) => Some(e),
            Error::MySql(e) => Some(e),
            Error::Sqlite(e) => Some(e),
            Error::Odbc(e) => Some(e),
            Error::Serde(e) => Some(e),
            Error::Msg(_) => None,
        }
    }
}
