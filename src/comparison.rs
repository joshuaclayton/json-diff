use super::Difference;
use serde_json::Value;

#[derive(Debug, PartialEq)]
pub enum Comparison<'a> {
    Same(&'a Value, &'a Value),
    Different(&'a Value, &'a Value, Difference<'a>),
}

pub fn compare<'a>(left: &'a Value, right: &'a Value) -> Comparison<'a> {
    match compare_values(left, right) {
        None => Comparison::Same(left, right),
        Some(otherwise) => Comparison::Different(left, right, otherwise),
    }
}

fn compare_values<'a>(left: &'a Value, right: &'a Value) -> Option<Difference<'a>> {
    if left == right {
        None
    } else {
        Some(compare_different_values(left, right))
    }
}

fn compare_different_values<'a>(left: &'a Value, right: &'a Value) -> Difference<'a> {
    match (left, right) {
        (Value::String(v1), Value::String(v2)) => Difference::MismatchedString(v1, v2),
        (Value::Number(v1), Value::Number(v2)) => Difference::MismatchedNumber(v1, v2),
        (Value::Bool(v1), Value::Bool(v2)) => Difference::MismatchedBool(*v1, *v2),
        (Value::Array(v1), Value::Array(v2)) => compare_arrays_of_values(v1, v2),
        (Value::Object(v1), Value::Object(v2)) => compare_maps(v1, v2),
        (_, _) => Difference::MismatchedTypes(left, right),
    }
}

fn compare_arrays_of_values<'a>(left: &'a Vec<Value>, right: &'a Vec<Value>) -> Difference<'a> {
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

    Difference::MismatchedArray(Box::new(differences))
}

fn compare_maps<'a>(
    left: &'a serde_json::Map<String, Value>,
    right: &'a serde_json::Map<String, Value>,
) -> Difference<'a> {
    let mut differences: Vec<Difference<'a>> = vec![];
    let mut left_keys: Vec<String> = vec![];

    for (key, left_value) in left {
        left_keys.push(key.to_string());
        match right.get(key) {
            None => differences.push(Difference::RemovedObjectKey(key, left_value)),
            Some(right_value) => match compare_values(left_value, right_value) {
                None => (),
                Some(otherwise) => {
                    differences.push(Difference::MismatchedObjectValue(key, Box::new(otherwise)))
                }
            },
        }
    }

    let mut right_keys = right.keys().collect::<Vec<&String>>();

    right_keys.retain(|&x| !left_keys.contains(x));

    for key in right_keys {
        differences.push(Difference::AddedObjectKey(key, right.get(key).unwrap()))
    }

    Difference::MismatchedObject(Box::new(differences))
}
