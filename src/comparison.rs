use super::lcs::{DiffComponent, LcsTable};
use super::Difference;
use serde_json::Value;

#[derive(Debug, PartialEq)]
pub enum Comparison<'a> {
    Same(&'a Value, &'a Value),
    Different(&'a Value, &'a Value, Difference<'a>),
}

#[derive(Debug, PartialEq)]
pub enum ObjectComparison<'a> {
    AddedObjectKey(&'a str, &'a Value),
    RemovedObjectKey(&'a str, &'a Value),
    MismatchedObjectValue(&'a str, Difference<'a>),
    Same(&'a str, &'a Value),
}

#[derive(Debug, PartialEq)]
pub enum ArrayComparison<'a> {
    ArrayDifference(usize, Difference<'a>),
    RemovedArrayValue(usize, &'a Value),
    AddedArrayValue(usize, &'a Value),
    Same(usize, &'a Value),
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
    let mut comparisons: Vec<ArrayComparison<'a>> = vec![];

    let table = LcsTable::new(left, right);
    let diff = table.diff();

    let mut index = 0;

    for chunk in diff.chunks(2) {
        let c1 = &chunk[0];

        match (c1, chunk.get(1)) {
            (DiffComponent::Deletion(v1), Some(DiffComponent::Insertion(v2))) => {
                comparisons.push(ArrayComparison::ArrayDifference(
                    index,
                    compare_different_values(v1, v2),
                ));

                index += 1;
            }

            (DiffComponent::Insertion(v1), Some(DiffComponent::Deletion(v2))) => {
                comparisons.push(ArrayComparison::ArrayDifference(
                    index,
                    compare_different_values(v2, v1),
                ));

                index += 1;
            }

            (v1, Some(v2)) => {
                comparisons.push(go(v1, &mut index));
                comparisons.push(go(v2, &mut index));
            }

            (v, None) => comparisons.push(go(v, &mut index)),
        };
    }

    Difference::MismatchedArray(comparisons)
}

fn go<'a>(diff: &DiffComponent<&'a Value>, index: &mut usize) -> ArrayComparison<'a> {
    match diff {
        DiffComponent::Insertion(v1) => {
            let result = ArrayComparison::AddedArrayValue(*index, v1);
            *index += 1;

            result
        }

        DiffComponent::Deletion(v1) => {
            let result = ArrayComparison::RemovedArrayValue(*index, v1);
            *index += 1;

            result
        }

        DiffComponent::Unchanged(_, v1) => {
            let result = ArrayComparison::Same(*index, v1);
            *index += 1;

            result
        }
    }
}

fn compare_maps<'a>(
    left: &'a serde_json::Map<String, Value>,
    right: &'a serde_json::Map<String, Value>,
) -> Difference<'a> {
    let mut comparisons: Vec<ObjectComparison<'a>> = vec![];
    let mut left_keys: Vec<String> = vec![];

    for (key, left_value) in left {
        left_keys.push(key.to_string());
        match right.get(key) {
            None => comparisons.push(ObjectComparison::RemovedObjectKey(key, left_value)),
            Some(right_value) => match compare_values(left_value, right_value) {
                None => comparisons.push(ObjectComparison::Same(key, right_value)),
                Some(otherwise) => {
                    comparisons.push(ObjectComparison::MismatchedObjectValue(key, otherwise))
                }
            },
        }
    }

    let mut right_keys = right.keys().collect::<Vec<&String>>();

    right_keys.retain(|&x| !left_keys.contains(x));

    for key in right_keys {
        comparisons.push(ObjectComparison::AddedObjectKey(
            key,
            right.get(key).unwrap(),
        ))
    }

    Difference::MismatchedObject(comparisons)
}
