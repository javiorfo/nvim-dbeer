use regex::Regex;

pub fn truncate_field_string<T: ToString>(field: T) -> String {
    let mut string = field.to_string();

    if string.lines().count() > 1 {
        string = format!("{}...", string.lines().next().unwrap_or_default());
    }

    if string.len() > 100 {
        string = format!("{}...", &string[..100]);
    }
    string
}

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

#[allow(dead_code)]
pub fn remove_sql_comments(sql: &str) -> String {
    sql.lines()
        .filter_map(|line| {
            let trimmed_line = line.trim_start();
            if trimmed_line.starts_with("--") {
                None
            } else if let Some(pos) = line.find("--") {
                Some(&line[..pos])
            } else {
                Some(line)
            }
        })
        .collect::<Vec<&str>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use crate::dbeer::query::{
        remove_sql_comments, split_queries, strip_sql_comments, truncate_field_string,
    };

    #[test]
    fn test_length_truncation() {
        let input = "Hello world";
        assert_eq!(truncate_field_string(input), "Hello world");

        let input = "a".repeat(110);
        let mut expected = "a".repeat(100);
        expected.push_str("...");
        assert_eq!(truncate_field_string(input), expected);
    }

    #[test]
    fn test_newline_truncation() {
        let input = "First line\nSecond line";
        assert_eq!(truncate_field_string(input), "First line...");

        let input = "First line\r\nSecond line";
        let result = truncate_field_string(input);
        assert_eq!(result, "First line...");

        let mut input = String::from("Short\n");
        input.push_str(&"a".repeat(110));
        assert_eq!(truncate_field_string(input), "Short...");
    }

    #[test]
    fn test_comment_at_end_of_line() {
        let sql = "SELECT * FROM users; -- Get all users";
        let expected = "SELECT * FROM users; ";
        assert_eq!(remove_sql_comments(sql), expected);
    }

    #[test]
    fn test_full_line_comment() {
        let sql = r#"
        create table testtable (
            -- This is a full-line comment
            id serial primary key,
            name text not null
        );"#;

        let expected = r#"
        create table testtable (
            id serial primary key,
            name text not null
        );"#;

        assert_eq!(remove_sql_comments(sql), expected);
    }

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
