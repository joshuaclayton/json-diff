use serde_json::{Result, Value};

fn main() {
    println!("Hello, world!");
}

fn compare<'a>(left: &'a Value, right: &'a Value) -> Comparison<'a> {
    match compare_values(left, right).as_slice() {
        [] => Comparison::Same(left, right),
        otherwise => Comparison::Different(left, right, otherwise.to_vec()),
    }
}

fn compare_values<'a>(left: &'a Value, right: &'a Value) -> Vec<Difference<'a>> {
    if left == right {
        vec![]
    } else {
        compare_different_values(left, right)
    }
}

fn compare_different_values<'a>(left: &'a Value, right: &'a Value) -> Vec<Difference<'a>> {
    match (left, right) {
        (Value::String(v1), Value::String(v2)) => vec![Difference::MismatchedString(v1, v2)],
        (Value::Number(v1), Value::Number(v2)) => vec![Difference::MismatchedNumber(v1, v2)],
        (Value::Bool(v1), Value::Bool(v2)) => vec![Difference::MismatchedBool(*v1, *v2)],
        (Value::Array(v1), Value::Array(v2)) => compare_arrays_of_values(v1, v2),
        (Value::Object(v1), Value::Object(v2)) => compare_maps(v1, v2),
        (_, _) => vec![Difference::MismatchedTypes(left, right)],
    }
}

fn compare_arrays_of_values<'a>(
    left: &'a Vec<Value>,
    right: &'a Vec<Value>,
) -> Vec<Difference<'a>> {
    let mut differences: Vec<Difference<'a>> = vec![];

    let mut rights = right.iter();
    for (index, left_item) in left.iter().enumerate() {
        match rights.next() {
            Some(right_item) => differences.extend(
                compare_values(left_item, right_item)
                    .into_iter()
                    .map(|v| Difference::ArrayDifference(index, Box::new(v)))
                    .collect::<Vec<_>>(),
            ),
            None => differences.push(Difference::RemovedArrayValue(index, left_item)),
        }
    }

    for (index, remainder) in rights.enumerate() {
        differences.push(Difference::AddedArrayValue(index + left.len(), remainder));
    }

    differences
}

fn compare_maps<'a>(
    left: &'a serde_json::Map<String, Value>,
    right: &'a serde_json::Map<String, Value>,
) -> Vec<Difference<'a>> {
    let mut differences: Vec<Difference<'a>> = vec![];
    let mut left_keys: Vec<String> = vec![];

    for (key, left_value) in left {
        left_keys.push(key.to_string());
        match right.get(key) {
            None => differences.push(Difference::RemovedObjectKey(key, left_value)),
            Some(right_value) => match compare_values(left_value, right_value).as_slice() {
                [] => (),
                otherwise => differences.push(Difference::MismatchedObjectValue(
                    key,
                    Box::new(otherwise.to_vec()),
                )),
            },
        }
    }

    let mut right_keys = right.keys().collect::<Vec<&String>>();

    right_keys.retain(|&x| !left_keys.contains(x));

    for key in right_keys {
        differences.push(Difference::AddedObjectKey(key, right.get(key).unwrap()))
    }

    differences
}

#[derive(Clone, Debug, PartialEq)]
enum Difference<'a> {
    Extra(serde_json::Map<String, Value>),
    Missing(serde_json::Map<String, Value>),
    MismatchedString(&'a str, &'a str),
    MismatchedNumber(&'a serde_json::Number, &'a serde_json::Number),
    MismatchedBool(bool, bool),
    MismatchedValue(&'a Value, &'a Value),
    MismatchedTypes(&'a Value, &'a Value),
    RemovedArrayValue(usize, &'a Value),
    AddedArrayValue(usize, &'a Value),
    MismatchedObjectValue(&'a str, Box<Vec<Difference<'a>>>),
    ArrayDifference(usize, Box<Difference<'a>>),
    RemovedObjectKey(&'a str, &'a Value),
    AddedObjectKey(&'a str, &'a Value),
}

#[derive(Debug, PartialEq)]
enum Comparison<'a> {
    Same(&'a Value, &'a Value),
    Different(&'a Value, &'a Value, Vec<Difference<'a>>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn compares_simple_structures() {
        assert_eq!(
            compare(&json("{\"foo\": \"bar\"}"), &json("{\"foo\": \"bar\"}")),
            Comparison::Same(&json("{\"foo\": \"bar\"}"), &json("{\"foo\": \"bar\"}"))
        );

        assert_eq!(
            compare(&json("\"foo\""), &json("\"baz\"")),
            Comparison::Different(
                &json("\"foo\""),
                &json("\"baz\""),
                vec![Difference::MismatchedString("foo", "baz")]
            )
        );

        assert_eq!(
            compare(&json("1.5"), &json("0")),
            Comparison::Different(
                &json("1.5"),
                &json("0"),
                vec![Difference::MismatchedNumber(
                    &serde_json::Number::from_f64(1.5).unwrap(),
                    &serde_json::Number::from(0)
                )]
            )
        );

        assert_eq!(
            compare(&json("1.5"), &json("\"baz\"")),
            Comparison::Different(
                &json("1.5"),
                &json("\"baz\""),
                vec![Difference::MismatchedTypes(&json("1.5"), &json("\"baz\""))]
            )
        );
    }

    #[test]
    fn compares_simple_arrays() {
        assert_eq!(
            compare(&json("[1, 2, 3, 4]"), &json("[1, 2, 3]")),
            Comparison::Different(
                &json("[1, 2, 3, 4]"),
                &json("[1, 2, 3]"),
                vec![Difference::RemovedArrayValue(3, &json("4")),]
            )
        );

        assert_eq!(
            compare(&json("[1, 2, 3]"), &json("[1, 2, 3, 4]")),
            Comparison::Different(
                &json("[1, 2, 3]"),
                &json("[1, 2, 3, 4]"),
                vec![Difference::AddedArrayValue(3, &json("4")),]
            )
        );

        assert_eq!(
            compare(&json("[1, 2, 3]"), &json("[1, 2, 4]")),
            Comparison::Different(
                &json("[1, 2, 3]"),
                &json("[1, 2, 4]"),
                vec![Difference::ArrayDifference(
                    2,
                    Box::new(Difference::MismatchedNumber(
                        &serde_json::Number::from(3),
                        &serde_json::Number::from(4)
                    ))
                )]
            )
        );

        assert_eq!(
            compare(&json("[1, 2, 3]"), &json("[\"1\", \"2\", \"4\"]")),
            Comparison::Different(
                &json("[1, 2, 3]"),
                &json("[\"1\", \"2\", \"4\"]"),
                vec![
                    Difference::ArrayDifference(
                        0,
                        Box::new(Difference::MismatchedTypes(&json("1"), &json("\"1\"")))
                    ),
                    Difference::ArrayDifference(
                        1,
                        Box::new(Difference::MismatchedTypes(&json("2"), &json("\"2\"")))
                    ),
                    Difference::ArrayDifference(
                        2,
                        Box::new(Difference::MismatchedTypes(&json("3"), &json("\"4\"")))
                    )
                ]
            )
        );
    }

    #[test]
    fn compare_simple_objects() {
        assert_eq!(
            compare(&json("{\"name\": \"Jane\"}"), &json("{\"name\": \"John\"}")),
            Comparison::Different(
                &json("{\"name\": \"Jane\"}"),
                &json("{\"name\": \"John\"}"),
                vec![Difference::MismatchedObjectValue(
                    "name",
                    Box::new(vec![Difference::MismatchedString("Jane", "John")])
                )]
            )
        );

        assert_eq!(
            compare(
                &json("{\"name\": \"Jane\", \"age\": 30}"),
                &json("{\"name\": \"John\"}")
            ),
            Comparison::Different(
                &json("{\"name\": \"Jane\", \"age\": 30}"),
                &json("{\"name\": \"John\"}"),
                vec![
                    Difference::RemovedObjectKey("age", &json("30")),
                    Difference::MismatchedObjectValue(
                        "name",
                        Box::new(vec![Difference::MismatchedString("Jane", "John")])
                    ),
                ]
            )
        );

        assert_eq!(
            compare(
                &json("{\"name\": \"Jane\"}"),
                &json("{\"name\": \"John\", \"age\": 30}")
            ),
            Comparison::Different(
                &json("{\"name\": \"Jane\"}"),
                &json("{\"name\": \"John\", \"age\": 30}"),
                vec![
                    Difference::MismatchedObjectValue(
                        "name",
                        Box::new(vec![Difference::MismatchedString("Jane", "John")])
                    ),
                    Difference::AddedObjectKey("age", &json("30")),
                ]
            )
        );

        assert_eq!(
            compare(
                &json("{\"name\": \"Jane\", \"dob\": \"01/01/1990\"}"),
                &json("{\"name\": \"John\", \"age\": 30}")
            ),
            Comparison::Different(
                &json("{\"name\": \"Jane\", \"dob\": \"01/01/1990\"}"),
                &json("{\"name\": \"John\", \"age\": 30}"),
                vec![
                    Difference::RemovedObjectKey("dob", &json("\"01/01/1990\"")),
                    Difference::MismatchedObjectValue(
                        "name",
                        Box::new(vec![Difference::MismatchedString("Jane", "John")])
                    ),
                    Difference::AddedObjectKey("age", &json("30")),
                ]
            )
        );
    }

    #[test]
    fn complex_example() {
        assert_eq!(
            compare(
                &json("[{\"person\": {\"name\": \"John\", \"age\": 31}}, {\"person\": {\"name\": \"Jane\", \"age\": 31}}]"),
                &json("[{\"person\": {\"name\": \"John\", \"age\": 30}}]")
            ),
            Comparison::Different(
                &json("[{\"person\": {\"name\": \"John\", \"age\": 31}}, {\"person\": {\"name\": \"Jane\", \"age\": 31}}]"),
                &json("[{\"person\": {\"name\": \"John\", \"age\": 30}}]"),


                vec![
                    Difference::ArrayDifference(0,
                        Box::new(Difference::MismatchedObjectValue("person",
                        Box::new(vec![
                            Difference::MismatchedObjectValue("age",
                                Box::new(vec![Difference::MismatchedNumber(
                                        &serde_json::Number::from(31),
                                        &serde_json::Number::from(30)
                                )])
                            )
                        ])),
                    )),
                    Difference::RemovedArrayValue(1, &json("{\"person\": {\"name\": \"Jane\", \"age\": 31}}")),
                ]
            )
        );
    }

    fn json(input: &str) -> Value {
        serde_json::from_str(input).unwrap()
    }
}
