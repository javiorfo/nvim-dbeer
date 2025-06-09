use regex::Regex;

pub fn is_select_query(query: &str) -> bool {
    query.trim().to_lowercase().starts_with("select")
}

pub fn split_queries(queries: &str) -> Vec<&str> {
    queries
        .split(";")
        .map(|query| query.trim())
        .filter(|&query| !query.is_empty())
        .collect::<Vec<_>>()
}

pub fn is_insert_update_or_delete(query: &str) -> bool {
    let query = query.trim().to_lowercase();
    query.contains("insert") || query.contains("update") || query.contains("delete")
}

pub fn strip_sql_comments(sql: &str) -> String {
    let re = Regex::new(
        r#"(?x)                           # verbose mode
        (                              # capture group 1: strings or comments
          ' (?: [^'] | '' )* '          # single quoted string, '' is escaped '
        | \" (?: [^\"] | \"\" )* \"     # double quoted string, "" is escaped "
        | /\* (?: .|\n)*? \*/           # block comment, non-greedy
        | -- [^\n]*                     # line comment
        )
        "#,
    )
    .unwrap();

    re.replace_all(sql, |caps: &regex::Captures| {
        let m = &caps[0];
        if m.starts_with('\'') || m.starts_with('\"') {
            m.to_string()
        } else {
            "".to_string()
        }
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use crate::dbeer::query::{split_queries, strip_sql_comments};

    #[test]
    fn test_strip() {
        let s = r#"-- comment
        select * from -- comment 
        -- table /* where */
        "#;

        assert!(!strip_sql_comments(s).contains("table"));
        assert!(!strip_sql_comments(s).contains("-- comment"));
        assert!(!strip_sql_comments(s).contains("/* where */"));
    }

    #[test]
    fn test_split_queries() {
        let s = r#"
        delete * from some where id = 1; 
        delete * from some2 where id = 1; 
        create table lala {
            id increment
        }
        "#;

        let split = split_queries(s);
        assert_eq!(3, split.len());
        assert_eq!("delete * from some2 where id = 1", split[1]);

        let s = "drop table dummies;";
        let split = split_queries(s);
        assert_eq!(1, split.len());
    }
}
