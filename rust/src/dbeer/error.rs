#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Postgres(postgres::Error),
    MySql(mysql::Error),
    Odbc(odbc::DiagnosticRecord),
    Msg(String),
}

pub type Result<T = ()> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "IO error => {}", e),
            Error::Postgres(e) => write!(f, "Postgres error => {}", e),
            Error::MySql(e) => write!(f, "MySql error => {}", e),
            Error::Odbc(e) => write!(f, "Odbc error => {}", e),
            Error::Msg(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::Postgres(e) => Some(e),
            Error::MySql(e) => Some(e),
            Error::Odbc(e) => Some(e),
            Error::Msg(_) => None,
        }
    }
}
