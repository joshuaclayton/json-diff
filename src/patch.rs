use crate::{ArrayComparison, Comparison, Difference, ObjectComparison};
use json_patch::patch;
use serde_json::value::Value;

pub fn generate_patch(comparison: &Comparison) -> Value {
    // println!("comparison: {:?}", comparison);
    match comparison {
        Comparison::Same(_, _) => Value::Array(vec![]),
        Comparison::Different(_, _, difference) => {
            let mut operations = vec![];
            let position = vec![];
            calculate_comparison(position, &mut operations, &difference);

            Value::Array(operations.into_iter().map(|v| v.into()).collect::<Vec<_>>())
        }
    }
}

enum Operation<'a> {
    Replace { path: Vec<String>, value: Value },
    Insert { path: Vec<String>, value: &'a Value },
    Remove { path: Vec<String> },
}

impl<'a> From<Operation<'a>> for Value {
    fn from(input: Operation) -> Value {
        match input {
            Operation::Replace { path, value } => {
                let mut inner = serde_json::Map::new();
                inner.insert("op".to_string(), "replace".into());
                inner.insert("path".to_string(), format!("/{}", path.join("/")).into());
                inner.insert("value".to_string(), value.clone());
                Value::Object(inner)
            }
            Operation::Insert { path, value } => {
                let mut inner = serde_json::Map::new();
                inner.insert("op".to_string(), "add".into());
                inner.insert("path".to_string(), format!("/{}", path.join("/")).into());
                inner.insert("value".to_string(), value.clone());
                Value::Object(inner)
            }
            Operation::Remove { path } => {
                let mut inner = serde_json::Map::new();
                inner.insert("op".to_string(), "remove".into());
                inner.insert("path".to_string(), format!("/{}", path.join("/")).into());
                Value::Object(inner)
            }
        }
    }
}

fn new_value<'a>(difference: &'a Difference<'a>) -> Value {
    match difference {
        Difference::MismatchedString(_, value) => Value::String(value.to_string()),
        _ => Value::Null,
    }
}

fn calculate_comparison<'a>(
    position: Vec<String>,
    operations: &mut Vec<Operation<'a>>,
    difference: &'a Difference<'a>,
) {
    match difference {
        Difference::MismatchedString(_, new_value) => {
            let path = position.clone();

            operations.push(Operation::Replace {
                path,
                value: Value::String(new_value.to_string()),
            })
        }
        Difference::MismatchedBool(_, new_value) => {
            let path = position.clone();

            operations.push(Operation::Replace {
                path,
                value: Value::Bool(*new_value),
            })
        }
        Difference::MismatchedNumber(_, new_value) => {
            let path = position.clone();

            operations.push(Operation::Replace {
                path,
                value: Value::Number(new_value.clone().clone()),
            })
        }
        Difference::MismatchedTypes(_, _) => (),
        Difference::MismatchedArray(array_comparisons) => {
            for array_comparison in array_comparisons {
                match array_comparison {
                    ArrayComparison::Same(_, _) => (),
                    ArrayComparison::RemovedArrayValue(index, _) => {
                        let mut path = position.clone();
                        path.push(index.to_string());

                        operations.push(Operation::Remove { path })
                    }
                    ArrayComparison::ArrayDifference(index, diff) => {
                        let mut path = position.clone();
                        path.push(index.to_string());
                        let value = new_value(&diff);

                        operations.push(Operation::Replace { path, value })
                    }
                    ArrayComparison::AddedArrayValue(index, value) => {
                        let mut temp = position.clone();
                        temp.push(index.to_string());

                        let insert = Operation::Insert { path: temp, value };
                        operations.push(insert)
                    }
                }
            }
        }
        Difference::MismatchedObject(object_comparisons) => {
            for object_comparison in object_comparisons {
                match object_comparison {
                    ObjectComparison::Same(_, _) => (),
                    ObjectComparison::AddedObjectKey(key, value) => {
                        let mut path = position.clone();
                        path.push(key.to_string());
                        let insert = Operation::Insert { path, value };

                        operations.push(insert)
                    }
                    ObjectComparison::RemovedObjectKey(key, _) => {
                        let mut path = position.clone();
                        path.push(key.to_string());
                        let remove = Operation::Remove { path };

                        operations.push(remove)
                    }
                    ObjectComparison::MismatchedObjectValue(key, diff) => {
                        let mut path = position.clone();
                        path.push(key.to_string());
                        calculate_comparison(path, operations, &diff)
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compare;
    use json_patch::patch;
    use serde_json::{from_str, json};

    #[test]
    fn generates_simple_patches() {
        let json1 = json!(
            { "name": "Jane Doe", "friends-count": 12, "dob": "2000-01-02", "admin": true, "hobbies": ["programming", "math"] }
        );
        let json2 = json!(
            { "name": "Jane Doe", "friends-count": 13, "admin": false, "dob": "2000-01-01", "hobbies": ["Rust"], "email": "jane@example.com" }
        );

        let comparison = compare(&json1, &json2);

        let generated_patch = json!([
            { "op": "replace", "path": "/admin", "value": false },
            { "op": "replace", "path": "/dob", "value": "2000-01-01" },
            { "op": "replace", "path": "/friends-count", "value": 13 },
            { "op": "replace", "path": "/hobbies/0", "value": "Rust" },
            { "op": "remove", "path": "/hobbies/1" },
            { "op": "add", "path": "/email", "value": "jane@example.com" },
        ]);

        assert_eq!(generate_patch(&comparison), generated_patch);
    }

    #[test]
    fn handles_array_insertions() {
        let json1 = json!(
            { "name": "Jane Doe", "dob": "2000-01-01", "hobbies": [] }
        );
        let json2 = json!(
            { "name": "Jane Doe", "dob": "2000-01-01", "hobbies": ["Rust"] }
        );

        let comparison = compare(&json1, &json2);

        let generated_patch = json!([
            { "op": "add", "path": "/hobbies/0", "value": "Rust" },
        ]);

        assert_eq!(generate_patch(&comparison), generated_patch);
    }

    #[test]
    fn generates_an_empty_list_when_no_differences_exist() {
        let json1 = json!(
            { "name": "Jane Doe", "dob": "2000-01-01", "hobbies": ["programming", "math"] }
        );

        let comparison = compare(&json1, &json1);

        let generated_patch = json!([]);

        assert_eq!(generate_patch(&comparison), generated_patch);
    }
}
