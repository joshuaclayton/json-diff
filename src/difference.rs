use super::{ArrayComparison, ObjectComparison};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq)]
pub enum Difference<'a> {
    MismatchedString(&'a str, &'a str),
    MismatchedNumber(&'a serde_json::Number, &'a serde_json::Number),
    MismatchedBool(bool, bool),
    MismatchedTypes(&'a Value, &'a Value),
    MismatchedArray(Vec<ArrayComparison<'a>>),
    MismatchedObject(Vec<ObjectComparison<'a>>),
}
