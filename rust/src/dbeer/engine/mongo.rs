use mongodb::{
    bson::{Document, to_document},
    sync::{Client, Collection},
};
use regex::Regex;

use crate::{dbeer, dbeer_debug};

#[derive(Debug)]
pub struct Mongo {
    function: Function,
    collection: Collection<Document>,
}

#[derive(Debug, PartialEq, Eq)]
enum Function {
    Find(String),
    Sort(String),  // TODO from find
    Skip(String),  // TODO from find
    Limit(String), // TODO from find
    CountDocuments(String),
    FindOne(String),
    InsertOne(String),
    InsertMany(String),
    DeleteOne(String),
    DeleteMany(String),
    UpdateOne(String),
    UpdateMany(String),
    Drop,
}

impl Function {
    #[allow(clippy::result_large_err)]
    pub fn from(value: &str) -> dbeer::Result<Self> {
        let re = Regex::new(r"^([^()]+)\(([^()]*)\)$")
            .map_err(|e| dbeer::Error::Msg(format!("Regex error: {}", e)))?;

        let caps = re.captures(value).ok_or_else(|| {
            dbeer::Error::Msg(format!("MongoDB function not supported: {}", value))
        })?;

        let function = &caps[1];
        let params = &caps[2];

        let params = params.to_string();
        let f = match function {
            "find" => Function::Find(params),
            "sort" => Function::Sort(params),
            "skip" => Function::Skip(params),
            "limit" => Function::Limit(params),
            "countDocuments" => Function::CountDocuments(params),
            "findOne" => Function::FindOne(params),
            "insertOne" => Function::InsertOne(params),
            "insertMany" => Function::InsertMany(params),
            "deleteOne" => Function::DeleteOne(params),
            "deleteMany" => Function::DeleteMany(params),
            "updateOne" => Function::UpdateOne(params),
            "updateMany" => Function::UpdateMany(params),
            "drop" => Function::Drop,
            _ => {
                return Err(dbeer::Error::Msg(format!(
                    "MongoDB function not supported: {}",
                    value
                )));
            }
        };

        Ok(f)
    }
}

impl Mongo {
    #[allow(clippy::result_large_err)]
    pub fn connect(conn_str: &str, db_name: &str, queries: &str) -> dbeer::Result<Self> {
        if queries.is_empty() {
            return Err(dbeer::Error::Msg("Nothing to execute".to_string()));
        }

        let client = Client::with_uri_str(conn_str).map_err(dbeer::Error::Mongo)?;
        let db = client.database(db_name);

        let parts: Vec<&str> = queries.split(".").collect();

        let (collection_name, function) = if parts.len() > 1 {
            (
                if parts[0] == "db" { parts[1] } else { parts[0] },
                Function::from(parts[parts.len() - 1])?,
            )
        } else {
            return Err(dbeer::Error::Msg(format!(
                "MongoDB bad format: {}",
                queries
            )));
        };

        let collection: Collection<Document> = db.collection(collection_name);

        Ok(Self {
            function,
            collection,
        })
    }

    #[allow(clippy::result_large_err)]
    pub fn run(&self, table: dbeer::Table) -> dbeer::Result {
        match &self.function {
            Function::Find(f) => {
                let filter = self.create_document(f)?;

                let cursor = self
                    .collection
                    .find(filter)
                    .run()
                    .map_err(dbeer::Error::Mongo)?;

                let mut results = Vec::new();
                for result in cursor {
                    let json_str =
                        serde_json::to_string_pretty(&result.map_err(dbeer::Error::Mongo)?)
                            .map_err(dbeer::Error::Serde)?;
                    results.push(json_str);
                }

                if results.is_empty() {
                    println!("  Query has returned 0 results.");
                    return Ok(());
                }

                let filepath = table.create_dbeer_mongo_file_format();
                println!("syn match dbeerStmtErr ' ' | hi link dbeerStmtErr ErrorMsg");
                println!("{filepath}");

                dbeer_debug!("File path: {filepath}. Results {results:#?}");

                return table.write_to_file(&filepath, &results);
            }

            Function::Sort(f)
            | Function::Skip(f)
            | Function::Limit(f)
            | Function::CountDocuments(f)
            | Function::FindOne(f)
            | Function::InsertOne(f)
            | Function::InsertMany(f)
            | Function::DeleteOne(f)
            | Function::DeleteMany(f)
            | Function::UpdateOne(f)
            | Function::UpdateMany(f) => {}
            Function::Drop => {}
        };

        Ok(())
    }

    #[allow(clippy::result_large_err)]
    fn create_document(&self, filter: &str) -> dbeer::Result<Document> {
        if filter.is_empty() {
            Ok(Document::new())
        } else {
            let json_value: serde_json::Value =
                serde_json::from_str(filter).map_err(dbeer::Error::Serde)?;

            Ok(to_document(&json_value).map_err(dbeer::Error::Bson)?)
        }
    }

    #[allow(clippy::result_large_err)]
    pub fn tables(&self) -> dbeer::Result {
        Ok(())
    }

    #[allow(clippy::result_large_err)]
    pub fn table_info(&self) -> dbeer::Result {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::dbeer::{self, engine::mongo::Function};

    #[test]
    fn test_from_function() {
        if let Ok(mf) = Function::from("find()") {
            assert_eq!(mf, Function::Find("".to_string()));
        }

        if let Ok(mf) = Function::from(r#"find({ "field": "value" })"#) {
            assert_eq!(mf, Function::Find(r#"{ "field": "value" }"#.to_string()));
        }
    }

    #[test]
    fn test_from_sort() {
        let input = r#"sort({"age": -1})"#;
        let expected = Function::Sort(r#"{"age": -1}"#.to_string());
        let result = Function::from(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_from_drop() {
        let input = "drop()";
        let expected = Function::Drop;
        let result = Function::from(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_from_drop_with_whitespace() {
        let input = "drop( )";
        let expected = Function::Drop;
        let result = Function::from(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_from_unsupported_function() {
        let input = r#"insertOne({"name": "John"})"#;
        let err = Function::from(input).unwrap_err();
        match err {
            dbeer::Error::Msg(msg) => {
                assert!(msg.contains("MongoDB function not supported"));
                assert!(msg.contains(input));
            }
            _ => panic!("Expected Error::Msg"),
        }
    }

    #[test]
    fn test_from_invalid_format_missing_parentheses() {
        let input = r#"find{"name": "John"}"#;
        let err = Function::from(input).unwrap_err();
        match err {
            dbeer::Error::Msg(msg) => {
                assert!(msg.contains("MongoDB function not supported"));
                assert!(msg.contains(input));
            }
            _ => panic!("Expected Error::Msg"),
        }
    }

    #[test]
    fn test_from_unsupported() {
        let input = "lookup";
        let err = Function::from(input).unwrap_err();
        match err {
            dbeer::Error::Msg(msg) => {
                assert!(msg.contains("MongoDB function not supported"));
                assert!(msg.contains(input));
            }
            _ => panic!("Expected Error::Msg"),
        }
    }
}
