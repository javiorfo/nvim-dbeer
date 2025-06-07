#[derive(Debug)]
pub enum DBeerError {
    Io(std::io::Error),
    Postgres(postgres::Error),
    Custom(String),
}

pub type Result<T = ()> = std::result::Result<T, DBeerError>;

impl std::fmt::Display for DBeerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DBeerError::Io(e) => write!(f, "IO error: {}", e),
            DBeerError::Postgres(e) => write!(f, "Postgres error: {}", e),
            DBeerError::Custom(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for DBeerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DBeerError::Io(e) => Some(e),
            DBeerError::Postgres(e) => Some(e),
            DBeerError::Custom(_) => None,
        }
    }
}
