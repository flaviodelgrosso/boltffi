pub struct NamingConvention;

impl NamingConvention {
    pub fn function_name(name: &str) -> String {
        Self::escape_keyword(name)
    }

    pub fn param_name(name: &str) -> String {
        Self::escape_keyword(name)
    }

    fn escape_keyword(name: &str) -> String {
        if Self::is_python_keyword(name) {
            format!("{name}_")
        } else {
            name.to_string()
        }
    }

    fn is_python_keyword(name: &str) -> bool {
        matches!(
            name,
            "False"
                | "None"
                | "True"
                | "and"
                | "as"
                | "assert"
                | "async"
                | "await"
                | "break"
                | "case"
                | "class"
                | "continue"
                | "def"
                | "del"
                | "elif"
                | "else"
                | "except"
                | "finally"
                | "for"
                | "from"
                | "global"
                | "if"
                | "import"
                | "in"
                | "is"
                | "lambda"
                | "match"
                | "nonlocal"
                | "not"
                | "or"
                | "pass"
                | "raise"
                | "return"
                | "try"
                | "type"
                | "while"
                | "with"
                | "yield"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::NamingConvention;

    #[test]
    fn escapes_python_keywords() {
        assert_eq!(NamingConvention::function_name("class"), "class_");
        assert_eq!(NamingConvention::function_name("match"), "match_");
        assert_eq!(NamingConvention::param_name("from"), "from_");
        assert_eq!(NamingConvention::param_name("type"), "type_");
    }

    #[test]
    fn leaves_non_keywords_unchanged() {
        assert_eq!(NamingConvention::function_name("echo_i32"), "echo_i32");
        assert_eq!(NamingConvention::param_name("value"), "value");
    }
}
