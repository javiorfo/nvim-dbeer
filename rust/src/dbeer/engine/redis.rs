use redis::{Client, Commands, Connection};

use crate::dbeer;

pub struct Redis {
    queries: String,
    connection: Connection,
}

impl Redis {
    #[allow(clippy::result_large_err)]
    pub fn connect(conn_str: &str, queries: &str) -> dbeer::Result<Self> {
        let client = Client::open(conn_str)?;
        Ok(Self {
            queries: queries.to_string(),
            connection: client.get_connection().map_err(|_| {
                dbeer::Error::Msg(format!(
                    "Error connecting Redis. Connection string: {conn_str}"
                ))
            })?,
        })
    }

    #[allow(clippy::result_large_err)]
    pub fn run(&mut self) -> dbeer::Result {
        Command::convert(&self.queries)?.execute(&mut self.connection)
    }
}

enum Command<'a> {
    Get(&'a str),
    Set { key: &'a str, value: &'a str },
    Del(Vec<&'a str>),
    Exists(&'a str),
    Keys(&'a str),
    Expire { key: &'a str, seconds: i64 },
    Ttl(&'a str),
    FlushAll,
}

impl<'a> Command<'a> {
    const GET: &'a str = "GET";
    const SET: &'a str = "SET";
    const DEL: &'a str = "DEL";
    const EXISTS: &'a str = "EXISTS";
    const EXPIRE: &'a str = "EXPIRE";
    const KEYS: &'a str = "KEYS";
    const TTL: &'a str = "TTL";
    const FLUSHALL: &'a str = "FLUSHALL";

    #[allow(clippy::result_large_err)]
    fn execute(&self, connection: &mut Connection) -> dbeer::Result {
        match self {
            Self::Get(key) => {
                let result: Option<String> = connection.get(key)?;
                println!("Key '{key}' is '{}'", result.unwrap_or("nil".to_string()));
                Ok(())
            }
            Self::Del(keys) => {
                let result: i32 = connection.del(keys)?;
                println!("{result} key(s) have been deleted.");
                Ok(())
            }
            Self::Expire { key, seconds } => {
                let _: () = connection.expire(key, *seconds)?;
                println!("Key '{key}' has been set with expiration of {seconds}s");
                Ok(())
            }
            Self::Set { key, value } => {
                let _: () = connection.set(key, value)?;
                println!("  Key '{key}' has been set with '{value}'");
                Ok(())
            }
            Self::Exists(key) => {
                let result: bool = connection.exists(key)?;
                println!(
                    "Key '{key}' {}",
                    if result { "exists." } else { "does not exist." }
                );
                Ok(())
            }
            Self::Keys(key) => {
                let result: Vec<String> = connection.keys(key)?;
                println!("Pattern '{key}' returns: [{}]", result.join(", "));
                Ok(())
            }
            Self::Ttl(key) => {
                let result: Option<String> = connection.ttl(key)?;
                println!(
                    "Key '{key}' remaining time is {}s",
                    result.unwrap_or("0".to_string())
                );
                Ok(())
            }
            Self::FlushAll => {
                let _: () = connection.flushall()?;
                println!("  All Keys have been deleted.");
                Ok(())
            }
        }
    }

    #[allow(clippy::result_large_err)]
    fn convert(queries: &'a str) -> dbeer::Result<Self> {
        let queries = queries.trim();

        if queries.starts_with(Self::GET) {
            return Ok(Self::Get(Self::take_key(queries, Self::GET)?));
        }

        if queries.starts_with(Self::TTL) {
            return Ok(Self::Ttl(Self::take_key(queries, Self::TTL)?));
        }

        if queries.starts_with(Self::KEYS) {
            return Ok(Self::Keys(Self::take_key(queries, Self::KEYS)?));
        }

        if queries.starts_with(Self::EXISTS) {
            return Ok(Self::Exists(Self::take_key(queries, Self::EXISTS)?));
        }

        if queries.eq(Self::FLUSHALL) {
            return Ok(Self::FlushAll);
        }

        if queries.starts_with(Self::DEL) {
            let keys = queries.strip_prefix(Self::DEL).unwrap().trim();
            return if keys.is_empty() {
                Err(dbeer::Error::Msg("keys is empty".to_string()))
            } else {
                let keys = keys.split(" ").map(|key| key.trim()).collect::<Vec<&str>>();

                Ok(Self::Del(keys))
            };
        }

        if queries.starts_with(Self::EXPIRE) {
            let values = queries.strip_prefix(Self::EXPIRE).unwrap().trim();
            return if values.is_empty() {
                Err(dbeer::Error::Msg("values are empty".to_string()))
            } else {
                let values = values
                    .split(" ")
                    .map(|key| key.trim())
                    .collect::<Vec<&str>>();

                let key = values
                    .first()
                    .ok_or(dbeer::Error::Msg("key is empty".to_string()))?;

                let seconds: i64 = values
                    .get(1)
                    .ok_or(dbeer::Error::Msg("Missing second value".to_string()))?
                    .parse()
                    .map_err(|_| {
                        dbeer::Error::Msg("Failed to parse seconds as number".to_string())
                    })?;

                Ok(Self::Expire { key, seconds })
            };
        }

        if queries.starts_with(Self::SET) {
            let values = queries.strip_prefix(Self::SET).unwrap().trim();
            return if values.is_empty() {
                Err(dbeer::Error::Msg("values are empty".to_string()))
            } else {
                let values = values
                    .splitn(2, " ")
                    .map(|key| key.trim())
                    .collect::<Vec<&str>>();

                let key = values
                    .first()
                    .ok_or(dbeer::Error::Msg("key is empty".to_string()))?;

                let value = values
                    .get(1)
                    .ok_or(dbeer::Error::Msg("Missing second value".to_string()))?
                    .trim_matches('"');

                Ok(Self::Set { key, value })
            };
        }

        Err(dbeer::Error::Msg(format!(
            "Command '{queries}' not supported!"
        )))
    }

    #[allow(clippy::result_large_err)]
    fn take_key(queries: &'a str, command: &'a str) -> dbeer::Result<&'a str> {
        let key = queries.strip_prefix(command).unwrap().trim();
        if key.is_empty() {
            Err(dbeer::Error::Msg("key is empty".to_string()))
        } else {
            Ok(key)
        }
    }
}
