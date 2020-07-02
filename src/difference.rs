use serde_json::Value;

#[derive(Clone, Debug, PartialEq)]
pub enum Difference<'a> {
    MismatchedString(&'a str, &'a str),
    MismatchedNumber(&'a serde_json::Number, &'a serde_json::Number),
    MismatchedBool(bool, bool),
    MismatchedTypes(&'a Value, &'a Value),
    MismatchedArray(Vec<ArrayDifference<'a>>),
    MismatchedObject(Vec<ObjectDifference<'a>>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ObjectDifference<'a> {
    AddedObjectKey(&'a str, &'a Value),
    RemovedObjectKey(&'a str, &'a Value),
    MismatchedObjectValue(&'a str, Difference<'a>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ArrayDifference<'a> {
    ArrayDifference(usize, Difference<'a>),
    RemovedArrayValue(usize, &'a Value),
    AddedArrayValue(usize, &'a Value),
}
