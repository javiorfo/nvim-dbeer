use mongodb::{
    bson::{Bson, Document, to_document},
    sync::{Client, Collection, Database},
};
use regex::Regex;
use serde_json::Value;

use crate::{dbeer, dbeer_debug};

#[derive(Debug, PartialEq, Eq)]
enum SubFunction {
    Sort(String),
    Skip(u64),
    Limit(i64),
    None,
}

#[derive(Debug, PartialEq, Eq)]
enum Function {
    Find(String, SubFunction),
    FindOne(String),
    CountDocuments(String),
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
    pub fn from(fn_str: &str, sub_fn_str: Option<&str>) -> dbeer::Result<Self> {
        let (function, params) = Self::get_function_and_params(fn_str)?;

        let f = match function.as_str() {
            "find" => match sub_fn_str {
                Some(f) => {
                    let (sub_function, sub_params) = Self::get_function_and_params(f)?;
                    Function::Find(
                        params,
                        match sub_function.as_str() {
                            "sort" => SubFunction::Sort(sub_params),
                            "skip" => SubFunction::Skip(sub_params.parse().map_err(|_| {
                                dbeer::Error::Msg(
                                    "skip parameter could not be cast to a number".to_string(),
                                )
                            })?),
                            "limit" => SubFunction::Limit(sub_params.parse().map_err(|_| {
                                dbeer::Error::Msg(
                                    "skip parameter could not be cast to a number".to_string(),
                                )
                            })?),
                            _ => SubFunction::None,
                        },
                    )
                }
                None => Function::Find(params, SubFunction::None),
            },
            "findOne" => Function::FindOne(params),
            "countDocuments" => Function::CountDocuments(params),
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
                    fn_str
                )));
            }
        };

        Ok(f)
    }

    #[allow(clippy::result_large_err)]
    fn get_function_and_params(value: &str) -> dbeer::Result<(String, String)> {
        let re = if value.contains("limit") || value.contains("skip") {
            Regex::new(r"^([a-zA-Z]+)\((\d+)\)$")
        } else {
            Regex::new(r"^([^()]+)\(([^()]*)\)$")
        }
        .map_err(|e| dbeer::Error::Msg(format!("Regex error: {}", e)))?;

        let caps = re.captures(value).ok_or_else(|| {
            dbeer::Error::Msg(format!("MongoDB function not supported: {}", value))
        })?;

        let function = &caps[1];
        let params = &caps[2];

        Ok((function.to_string(), params.to_string()))
    }
}

#[derive(Debug)]
pub struct Mongo {
    queries: String,
    database: Database,
}

impl Mongo {
    #[allow(clippy::result_large_err)]
    pub fn connect(conn_str: &str, db_name: &str, queries: &str) -> dbeer::Result<Self> {
        let client = Client::with_uri_str(conn_str).map_err(dbeer::Error::Mongo)?;
        let database = client.database(db_name);

        Ok(Self {
            queries: queries.to_string(),
            database,
        })
    }

    #[allow(clippy::result_large_err)]
    pub fn run(&self, table: dbeer::Table) -> dbeer::Result {
        let parts: Vec<&str> = self.queries.split(".").collect();

        let (collection_name, function) = if parts.len() > 1 {
            if parts[0] == "db" {
                (
                    parts[1],
                    Function::from(
                        parts.get(2).ok_or_else(|| {
                            dbeer::Error::Msg(format!("Error parsing function {:?}", parts))
                        })?,
                        parts.get(3).copied(),
                    )?,
                )
            } else {
                (parts[0], Function::from(parts[1], parts.get(2).copied())?)
            }
        } else {
            return Err(dbeer::Error::Msg(format!(
                "MongoDB bad format: {}",
                self.queries
            )));
        };

        let collection: Collection<Document> = self.database.collection(collection_name);

        match &function {
            Function::Find(params, sub_function) => {
                let cursor = match sub_function {
                    SubFunction::Sort(sub_params) => collection
                        .find(self.create_document(params)?)
                        .sort(self.create_document(sub_params)?)
                        .run()
                        .map_err(dbeer::Error::Mongo)?,
                    SubFunction::Skip(number) => collection
                        .find(self.create_document(params)?)
                        .skip(*number)
                        .run()
                        .map_err(dbeer::Error::Mongo)?,
                    SubFunction::Limit(number) => collection
                        .find(self.create_document(params)?)
                        .limit(*number)
                        .run()
                        .map_err(dbeer::Error::Mongo)?,
                    _ => collection
                        .find(self.create_document(params)?)
                        .run()
                        .map_err(dbeer::Error::Mongo)?,
                };

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

                table.create_execute_result_file(dbeer::Format::Json(results))?
            }
            Function::FindOne(params) => {
                let document = collection
                    .find_one(self.create_document(params)?)
                    .run()
                    .map_err(dbeer::Error::Mongo)?;

                if document.is_none() {
                    println!("  Query has returned 0 results.");
                    return Ok(());
                }

                let json_str = serde_json::to_string_pretty(&document.unwrap())
                    .map_err(dbeer::Error::Serde)?;

                table.create_execute_result_file(dbeer::Format::Json(vec![json_str]))?
            }
            Function::CountDocuments(params) => {
                let total = collection
                    .count_documents(self.create_document(params)?)
                    .run()
                    .map_err(dbeer::Error::Mongo)?;

                println!(
                    "  Collection {} count: {} results.",
                    collection.name(),
                    total
                );

                return Ok(());
            }
            Function::InsertOne(params) => {
                let inserted = collection
                    .insert_one(self.create_document(params)?)
                    .run()
                    .map_err(dbeer::Error::Mongo)?
                    .inserted_id
                    .as_object_id()
                    .unwrap();

                println!(
                    "  Collection {}, document inserted with ID: {}",
                    collection.name(),
                    inserted
                );

                return Ok(());
            }
            Function::DeleteOne(params) => {
                let deleted = collection
                    .delete_one(self.create_document(params)?)
                    .run()
                    .map_err(dbeer::Error::Mongo)?
                    .deleted_count;

                println!(
                    "  Collection {}, deleted {} document(s)",
                    collection.name(),
                    deleted
                );

                return Ok(());
            }
            Function::UpdateOne(params) => {
                let (query, set) = Self::get_query_and_set(params)?;

                let modified = collection
                    .update_one(self.create_document(&query)?, self.create_document(&set)?)
                    .run()
                    .map_err(dbeer::Error::Mongo)?
                    .modified_count;

                println!(
                    "  Collection {}, updated {} document(s)",
                    collection.name(),
                    modified
                );

                return Ok(());
            }
            Function::InsertMany(params) => {
                let parsed_json: Value =
                    serde_json::from_str(params).map_err(dbeer::Error::Serde)?;

                let docs: Vec<Document> = parsed_json
                    .as_array()
                    .ok_or_else(|| dbeer::Error::Msg("Expected JSON array".to_string()))?
                    .iter()
                    .map(|v| to_document(v).expect("Failed to convert to BSON document"))
                    .collect();

                let inserted = collection
                    .insert_many(docs)
                    .run()
                    .map_err(dbeer::Error::Mongo)?
                    .inserted_ids
                    .values()
                    .map(|bson| match bson {
                        Bson::ObjectId(oid) => oid.to_hex(),
                        other => other.to_string(),
                    })
                    .collect::<Vec<String>>()
                    .join(", ");

                println!(
                    "  Collection {}, documents inserted with ID(s): {}",
                    collection.name(),
                    inserted
                );

                return Ok(());
            }
            Function::DeleteMany(params) => {
                let deleted = collection
                    .delete_many(self.create_document(params)?)
                    .run()
                    .map_err(dbeer::Error::Mongo)?
                    .deleted_count;

                println!(
                    "  Collection {}, deleted {} document(s)",
                    collection.name(),
                    deleted
                );

                return Ok(());
            }
            Function::UpdateMany(params) => {
                let (query, set) = Self::get_query_and_set(params)?;

                let modified = collection
                    .update_many(self.create_document(&query)?, self.create_document(&set)?)
                    .run()
                    .map_err(dbeer::Error::Mongo)?
                    .modified_count;

                println!(
                    "  Collection {}, updated {} document(s)",
                    collection.name(),
                    modified
                );

                return Ok(());
            }
            Function::Drop => {
                collection.drop().run().map_err(dbeer::Error::Mongo)?;

                println!("  Collection {} dropped successfully.", collection.name(),);
            }
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
    fn get_query_and_set(filter: &str) -> dbeer::Result<(String, String)> {
        Ok(match filter.find("$set") {
            Some(set_pos) => {
                let start_second_json = filter[..set_pos].rfind('{').unwrap_or(0);

                let first_json = &filter[..start_second_json]
                    .trim()
                    .trim_end_matches(',')
                    .to_string();
                let second_json = &filter[start_second_json..].trim().to_string();

                (first_json.clone(), second_json.clone())
            }
            _ => return Err(dbeer::Error::Msg("$set not found in update".to_string())),
        })
    }

    #[allow(clippy::result_large_err)]
    pub fn tables(&self) -> dbeer::Result {
        let collection_names = self
            .database
            .list_collections()
            .run()
            .map_err(dbeer::Error::Mongo)?
            .map(|ce| ce.unwrap().name.to_uppercase())
            .collect::<Vec<String>>()
            .join(" ");

        dbeer_debug!("Table names: {collection_names}");
        println!("[{collection_names}]");

        Ok(())
    }

    #[allow(clippy::result_large_err)]
    pub fn table_info(&self) -> dbeer::Result {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::dbeer::{
        self,
        engine::mongo::{Function, SubFunction},
    };

    #[test]
    fn test_from_function() {
        if let Ok(mf) = Function::from("find()", None) {
            assert_eq!(mf, Function::Find("".to_string(), SubFunction::None));
        }

        if let Ok(mf) = Function::from(r#"find({ "field": "value" }).sort({ "field": -1 })"#, None)
        {
            assert_eq!(
                mf,
                Function::Find(
                    r#"{ "field": "value" }"#.to_string(),
                    SubFunction::Sort(r#"{ "field": -1 }"#.to_string())
                )
            );
        }

        if let Ok(mf) = Function::from(r#"find().limit(10)"#, None) {
            assert_eq!(mf, Function::Find("".to_string(), SubFunction::Limit(10)));
        }

        if let Ok(mf) = Function::from(r#"find({ "field": "value" }).skip(2)"#, None) {
            assert_eq!(mf, Function::Find("".to_string(), SubFunction::Skip(2)));
        }
    }

    #[test]
    fn test_from_drop() {
        let input = "drop()";
        let expected = Function::Drop;
        let result = Function::from(input, None).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_from_drop_with_whitespace() {
        let input = "drop( )";
        let expected = Function::Drop;
        let result = Function::from(input, None).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_from_update_one() {
        let input = r#"updateOne({ "price": 99 }, { $set: { "price": 98 } })"#;
        let expected =
            Function::UpdateOne(r#"{ "price": 99 }, { $set: { "price": 98 } }"#.to_string());
        let result = Function::from(input, None).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_from_unsupported_function() {
        let input = r#"replace({"name": "John"})"#;
        let err = Function::from(input, None).unwrap_err();
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
        let err = Function::from(input, None).unwrap_err();
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
        let err = Function::from(input, None).unwrap_err();
        match err {
            dbeer::Error::Msg(msg) => {
                assert!(msg.contains("MongoDB function not supported"));
                assert!(msg.contains(input));
            }
            _ => panic!("Expected Error::Msg"),
        }
    }
}
