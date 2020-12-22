use crate::json_selector;
use serde_json;

pub enum Operation<'a> {
    Add {
        path: &'a str,
        value: serde_json::Value,
    },
    Remove {
        path: &'a str,
    },
    Replace {
        path: &'a str,
        value: serde_json::Value,
    },
    Move {
        path: &'a str,
        from: &'a str,
    },

    Copy {
        path: &'a str,
        from: &'a str,
    },
    Test {
        path: &'a str,
        value: serde_json::Value,
    },
}

#[derive(Debug, PartialEq)]
pub enum OperationError<'a> {
    MissingKeyForSelector(&'a str),
    FailedTest(&'a str),
    InvalidKey(&'a str),
    DisallowedMove(&'a str),
    InvalidIndex(&'a str),
}

impl<'a> Operation<'a> {
    pub fn apply(
        json: serde_json::Value,
        operation: &Operation<'a>,
    ) -> Result<serde_json::Value, OperationError<'a>> {
        match operation {
            Operation::Test { path, value } => {
                if let Some(found_value) = json_selector::value_at(&json, path).ok() {
                    if &found_value == value {
                        Ok(json)
                    } else {
                        Err(OperationError::FailedTest(path))
                    }
                } else {
                    Err(OperationError::InvalidKey(path))
                }
            }
            _ => Ok(json),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn add_to_existing() {
        let base = json("{\"a\": 1}");
        assert_eq!(
            Operation::apply(
                base,
                &Operation::Add {
                    path: "/b",
                    value: json("[1, 2, {\"c\": 3}]")
                }
            ),
            Ok(json("{\"a\": 1, \"b\": [1, 2, {\"c\": 3}]}"))
        );
    }

    #[test]
    fn add_to_array_at_index() {
        let base = json("{\"a\": [1, 2]}");
        assert_eq!(
            Operation::apply(
                base,
                &Operation::Add {
                    path: "/a/1",
                    value: json("[1, 2, {\"c\": 3}]")
                }
            ),
            Ok(json("{\"a\": [1, [1, 2, {\"c\": 3}], 2]}"))
        );
    }

    #[test]
    fn add_to_nested_object() {
        let base = json("{\"a\": {\"current\": true}}");
        assert_eq!(
            Operation::apply(
                base,
                &Operation::Add {
                    path: "/a/new",
                    value: json("\"also true\"")
                }
            ),
            Ok(json("{\"a\": {\"current\": true, \"new\": \"also true\"}}"))
        );
    }

    #[test]
    fn add_to_nested_object_replaces() {
        let base = json("{\"a\": {\"current\": true}}");
        assert_eq!(
            Operation::apply(
                base,
                &Operation::Add {
                    path: "/a/current",
                    value: json("\"also true\"")
                }
            ),
            Ok(json("{\"a\": {\"current\": \"also true\"}}"))
        );
    }

    #[test]
    fn add_to_end_of_array() {
        let base = json("{\"a\": [1, 2, 3]}");
        assert_eq!(
            Operation::apply(
                base,
                &Operation::Add {
                    path: "/a/-",
                    value: json("\"also true\"")
                }
            ),
            Ok(json("{\"a\": [1, 2, 3, \"also true\"]}"))
        );
    }

    #[test]
    fn fails_when_accessing_past_array() {
        let base = json("{\"a\": [1, 2, 3]}");
        assert_eq!(
            Operation::apply(
                base,
                &Operation::Add {
                    path: "/a/4",
                    value: json("\"also true\"")
                }
            ),
            Err(OperationError::InvalidIndex("/a/4"))
        );
    }

    #[test]
    fn copy_succeeds() {
        let base = json("{\"a\": 1}");
        assert_eq!(
            Operation::apply(
                base,
                &Operation::Copy {
                    path: "/b",
                    from: "/a"
                }
            ),
            Ok(json("{\"a\": 1, \"b\": 1}"))
        );
    }

    #[test]
    fn remove_succeeds() {
        let base = json("{\"a\": 1}");
        assert_eq!(
            Operation::apply(base, &Operation::Remove { path: "/a" }),
            Ok(json("{}"))
        );
    }

    #[test]
    fn remove_fails() {
        let base = json("{\"a\": 1}");
        assert_eq!(
            Operation::apply(base, &Operation::Remove { path: "/b" }),
            Err(OperationError::InvalidKey("/b"))
        );
    }

    #[test]
    fn replace_succeeds() {
        let base = json("{\"a\": 1}");
        assert_eq!(
            Operation::apply(
                base,
                &Operation::Replace {
                    path: "/a",
                    value: json("{\"b\": [1]}")
                }
            ),
            Ok(json("{\"a\": {\"b\": [1]}}"))
        );
    }

    #[test]
    fn copy_fails() {
        let base = json("{\"a\": 1}");
        assert_eq!(
            Operation::apply(
                base,
                &Operation::Move {
                    path: "/b",
                    from: "/c"
                }
            ),
            Err(OperationError::InvalidKey("/c"))
        );
    }

    #[test]
    fn move_succeeds() {
        let base = json("{\"a\": 1}");
        assert_eq!(
            Operation::apply(
                base,
                &Operation::Move {
                    path: "/b",
                    from: "/a"
                }
            ),
            Ok(json("{\"b\": 1"))
        );
    }

    #[test]
    fn move_fails_due_to_further_depth() {
        let base = json("{\"a\": 1}");
        assert_eq!(
            Operation::apply(
                base,
                &Operation::Copy {
                    path: "/a/c",
                    from: "/a"
                }
            ),
            Err(OperationError::DisallowedMove("/a/c"))
        );
    }

    #[test]
    fn move_fails_due_invalid_source() {
        let base = json("{\"a\": 1}");
        assert_eq!(
            Operation::apply(
                base,
                &Operation::Move {
                    path: "/c",
                    from: "/b"
                }
            ),
            Err(OperationError::InvalidKey("/c"))
        );
    }

    #[test]
    fn test_succeeds() {
        let base = json("{\"a\": 1}");
        assert_eq!(
            Operation::apply(
                base,
                &Operation::Test {
                    path: "/a",
                    value: json("1")
                }
            ),
            Ok(json("{\"a\": 1}"))
        );
    }

    #[test]
    fn test_fails() {
        let base = json("{\"a\": 1}");
        assert_eq!(
            Operation::apply(
                base,
                &Operation::Test {
                    path: "/a",
                    value: json("2")
                }
            ),
            Err(OperationError::FailedTest("/a"))
        );
    }

    #[test]
    fn test_fails_for_invalid_selector() {
        let base = json("{\"a\": 1}");
        assert_eq!(
            Operation::apply(
                base,
                &Operation::Test {
                    path: "/b",
                    value: json("2")
                }
            ),
            Err(OperationError::InvalidKey("/b"))
        );
    }

    fn json(input: &str) -> Value {
        serde_json::from_str(input).unwrap()
    }
}
