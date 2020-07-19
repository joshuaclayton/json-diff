use colored::*;
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
            print_comparison(0, &comparison);
        }
        _ => {
            eprintln!("Failed parsing JSON");
            std::process::exit(1)
        }
    }

    Ok(())
}

fn print_comparison(depth: usize, comp: &Comparison) {
    match comp {
        Comparison::Same(v1, _) => println!("{}", v1),
        Comparison::Different(_, _, d) => print_difference(depth, d),
    }
}

fn print_difference(depth: usize, diff: &Difference) {
    let padding = format!("{:width$}", "", width = depth * 2);
    match diff {
        Difference::MismatchedString(v1, v2) => print!(
            "{}\"{}{}\"",
            padding,
            format!("{}", v1).red().strikethrough(),
            format!("{}", v2).green()
        ),
        Difference::MismatchedNumber(v1, v2) => print!(
            "{}{}{}",
            padding,
            format!("{}", v1).red().strikethrough(),
            format!("{}", v2).green()
        ),
        Difference::MismatchedBool(v1, v2) => print!(
            "{}{}{}",
            padding,
            format!("{}", v1).red().strikethrough(),
            format!("{}", v2).green()
        ),
        Difference::MismatchedTypes(v1, v2) => print!(
            "Expected {}, got {}",
            json_type_name(&v1),
            json_type_name(&v2)
        ),
        Difference::MismatchedArray(xs) => {
            println!("[");
            for x in xs {
                print_array_comparison(depth + 1, &x);
            }
            println!("{}]", padding);
        }
        Difference::MismatchedObject(xs) => {
            println!("{}{{", padding);
            for x in xs {
                print_object_comparison(depth + 1, &x);
            }
            print!("{}}}", padding);
        }
    }
}

fn print_array_comparison(depth: usize, diff: &ArrayComparison) {
    let padding = format!("{:width$}", "", width = depth * 2);
    match diff {
        ArrayComparison::ArrayDifference(_, d) => {
            print_difference(depth, d);
            println!(",");
        }

        ArrayComparison::RemovedArrayValue(_, v) => {
            println!("{}{},", padding, format!("{}", v).red().strikethrough())
        }

        ArrayComparison::AddedArrayValue(_, v) => {
            println!("{}{},", padding, format!("{}", v).green())
        }
        ArrayComparison::Same(_, v) => println!("{}{},", padding, v),
    }
}

fn print_object_comparison(depth: usize, diff: &ObjectComparison) {
    let padding = format!("{:width$}", "", width = depth * 2);
    match diff {
        ObjectComparison::MismatchedObjectValue(k, d) => {
            print!("{}\"{}\": ", padding, k.yellow());
            print_difference(depth, d);
            println!(",");
        }
        ObjectComparison::RemovedObjectKey(k, v) => println!(
            "{}{},",
            padding,
            format!("\"{}\": {}", k, v).red().strikethrough()
        ),
        ObjectComparison::AddedObjectKey(k, v) => {
            println!("{}{},", padding, format!("\"{}\": {}", k, v).green())
        }
        ObjectComparison::Same(k, v) => println!("{}\"{}\": {},", padding, k, v),
    }
}

fn json_type_name(json: &Value) -> String {
    match json {
        Value::Number(_) => "number".to_string(),
        Value::String(_) => "string".to_string(),
        Value::Bool(_) => "bool".to_string(),
        Value::Null => "null".to_string(),
        Value::Object(_) => "object".to_string(),
        Value::Array(_) => "array".to_string(),
    }
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
    }

    #[test]
    fn handles_complex_array_interactions() {
        assert_eq!(
            compare(&json("[1, 2, 3, 4, 5, 8]"), &json("[1, 2, 4, 5, 6, 7, 8]")),
            Comparison::Different(
                &json("[1, 2, 3, 4, 5, 8]"),
                &json("[1, 2, 4, 5, 6, 7, 8]"),
                Difference::MismatchedArray(vec![
                    ArrayComparison::Same(0, &json("1")),
                    ArrayComparison::Same(1, &json("2")),
                    ArrayComparison::RemovedArrayValue(2, &json("3")),
                    ArrayComparison::Same(3, &json("4")),
                    ArrayComparison::Same(4, &json("5")),
                    ArrayComparison::AddedArrayValue(5, &json("6")),
                    ArrayComparison::AddedArrayValue(6, &json("7")),
                    ArrayComparison::Same(7, &json("8")),
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
