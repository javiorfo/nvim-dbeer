use regex::Regex;

pub fn is_select_query(query: &str) -> bool {
    let query = query.trim();
    !query.is_empty() && query.to_lowercase().starts_with("select")
}

pub fn strip_sql_comments(sql: &str) -> String {
    sql.lines()
        .map(|line| match line.find("--") {
            Some(pos) => &line[..pos],
            _ => line,
        })
        .collect::<Vec<&str>>()
        .join(" ")
}

fn strip_sql_comments_regex(sql: &str) -> String {
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
    use crate::dbeer::query::strip_sql_comments_regex;

    #[test]
    fn test_strip() {
        let s = r#"-- comment
        select * from -- comment 
        -- table /* where */
        "#;

        assert!(!strip_sql_comments_regex(s).contains("table"));
        assert!(!strip_sql_comments_regex(s).contains("-- comment"));
        assert!(!strip_sql_comments_regex(s).contains("/* where */"));
    }
}
