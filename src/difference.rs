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

#[derive(Clone, Debug, PartialEq)]
pub enum ObjectComparison<'a> {
    AddedObjectKey(&'a str, &'a Value),
    RemovedObjectKey(&'a str, &'a Value),
    MismatchedObjectValue(&'a str, Difference<'a>),
    Same(&'a str, &'a Value),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ArrayComparison<'a> {
    ArrayDifference(usize, Difference<'a>),
    RemovedArrayValue(usize, &'a Value),
    AddedArrayValue(usize, &'a Value),
    Same(usize, &'a Value),
}
