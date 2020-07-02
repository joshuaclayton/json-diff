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
