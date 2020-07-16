use json_diff::*;
use serde_json::Value;
use std::env;
use std::fs;
use std::path::Path;

fn main() -> Result<(), error::Error> {
    let args: Vec<String> = env::args().collect();

    let file1 = &args[1];
    let file2 = &args[2];
    match (read_as_json(file1), read_as_json(file2)) {
        (Ok(file1_body), Ok(file2_body)) => {
            let comparison = compare(&file1_body, &file2_body);
            println!("{:?}", comparison);
        }
        _ => {
            eprintln!("Failed parsing JSON");
            std::process::exit(1)
        }
    }

    Ok(())
}

fn read_as_json<P: AsRef<Path>>(filename: P) -> Result<Value, error::Error> {
    let contents = fs::read_to_string(filename)?;

    serde_json::from_str(&contents).map_err(|v| v.into())
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
                Difference::MismatchedString("foo", "baz")
            )
        );

        assert_eq!(
            compare(&json("1.5"), &json("0")),
            Comparison::Different(
                &json("1.5"),
                &json("0"),
                Difference::MismatchedNumber(
                    &serde_json::Number::from_f64(1.5).unwrap(),
                    &serde_json::Number::from(0)
                )
            )
        );

        assert_eq!(
            compare(&json("1.5"), &json("\"baz\"")),
            Comparison::Different(
                &json("1.5"),
                &json("\"baz\""),
                Difference::MismatchedTypes(&json("1.5"), &json("\"baz\""))
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
                Difference::MismatchedArray(vec![
                    ArrayComparison::Same(0, &json("1")),
                    ArrayComparison::Same(1, &json("2")),
                    ArrayComparison::Same(2, &json("3")),
                    ArrayComparison::RemovedArrayValue(3, &json("4"))
                ])
            )
        );

        assert_eq!(
            compare(&json("[1, 2, 3]"), &json("[1, 2, 3, 4]")),
            Comparison::Different(
                &json("[1, 2, 3]"),
                &json("[1, 2, 3, 4]"),
                Difference::MismatchedArray(vec![
                    ArrayComparison::Same(0, &json("1")),
                    ArrayComparison::Same(1, &json("2")),
                    ArrayComparison::Same(2, &json("3")),
                    ArrayComparison::AddedArrayValue(3, &json("4")),
                ])
            )
        );

        assert_eq!(
            compare(&json("[1, 2, 3]"), &json("[1, 2, 4]")),
            Comparison::Different(
                &json("[1, 2, 3]"),
                &json("[1, 2, 4]"),
                Difference::MismatchedArray(vec![
                    ArrayComparison::Same(0, &json("1")),
                    ArrayComparison::Same(1, &json("2")),
                    ArrayComparison::ArrayDifference(
                        2,
                        Difference::MismatchedNumber(
                            &serde_json::Number::from(3),
                            &serde_json::Number::from(4)
                        )
                    )
                ])
            )
        );

        assert_eq!(
            compare(&json("[1, 2, 3]"), &json("[\"1\", \"2\", \"4\"]")),
            Comparison::Different(
                &json("[1, 2, 3]"),
                &json("[\"1\", \"2\", \"4\"]"),
                Difference::MismatchedArray(vec![
                    ArrayComparison::ArrayDifference(
                        0,
                        Difference::MismatchedTypes(&json("1"), &json("\"1\""))
                    ),
                    ArrayComparison::ArrayDifference(
                        1,
                        Difference::MismatchedTypes(&json("2"), &json("\"2\""))
                    ),
                    ArrayComparison::ArrayDifference(
                        2,
                        Difference::MismatchedTypes(&json("3"), &json("\"4\""))
                    )
                ])
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
                Difference::MismatchedObject(vec![ObjectComparison::MismatchedObjectValue(
                    "name",
                    Difference::MismatchedString("Jane", "John")
                )])
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
                Difference::MismatchedObject(vec![
                    ObjectComparison::RemovedObjectKey("age", &json("30")),
                    ObjectComparison::MismatchedObjectValue(
                        "name",
                        Difference::MismatchedString("Jane", "John")
                    ),
                ])
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
                Difference::MismatchedObject(vec![
                    ObjectComparison::MismatchedObjectValue(
                        "name",
                        Difference::MismatchedString("Jane", "John")
                    ),
                    ObjectComparison::AddedObjectKey("age", &json("30")),
                ])
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
                Difference::MismatchedObject(vec![
                    ObjectComparison::RemovedObjectKey("dob", &json("\"01/01/1990\"")),
                    ObjectComparison::MismatchedObjectValue(
                        "name",
                        Difference::MismatchedString("Jane", "John")
                    ),
                    ObjectComparison::AddedObjectKey("age", &json("30")),
                ])
            )
        );
    }

    #[test]
    fn complex_example() {
        let removed_json = json("{\"person\": {\"name\": \"Jane\", \"age\": 31}}");
        let removed_array = ArrayComparison::RemovedArrayValue(1, &removed_json);

        assert_eq!(
            compare(
                &json("[{\"person\": {\"name\": \"John\", \"age\": 31}}, {\"person\": {\"name\": \"Jane\", \"age\": 31}}]"),
                &json("[{\"person\": {\"name\": \"John\", \"age\": 30}}]")
            ),
            Comparison::Different(
                &json("[{\"person\": {\"name\": \"John\", \"age\": 31}}, {\"person\": {\"name\": \"Jane\", \"age\": 31}}]"),
                &json("[{\"person\": {\"name\": \"John\", \"age\": 30}}]"),
                Difference::MismatchedArray(vec![
                    ArrayComparison::ArrayDifference(
                        0,
                        Difference::MismatchedObject(vec![
                            ObjectComparison::MismatchedObjectValue(
                                "person",
                                Difference::MismatchedObject(vec![
                                    ObjectComparison::MismatchedObjectValue(
                                        "age",
                                        Difference::MismatchedNumber(
                                            &serde_json::Number::from(31),
                                            &serde_json::Number::from(30),
                                        ),
                                    ),
                                    ObjectComparison::Same("name", &serde_json::Value::String("John".to_string())),
                                ])
                            ),
                        ]),
                    ),
                    removed_array,
                ])
            )
        );
    }

    fn json(input: &str) -> Value {
        serde_json::from_str(input).unwrap()
    }
}
