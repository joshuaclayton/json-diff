use serde_json::Value;

#[derive(Clone, Debug, PartialEq)]
pub enum Difference<'a> {
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
