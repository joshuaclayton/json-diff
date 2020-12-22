use nom::{
    branch::alt,
    bytes::complete::{tag, take_till1},
    character::complete::digit1,
    combinator::{eof, map, map_res, recognize},
    multi::separated_list1,
    sequence::preceded,
    IResult,
};
use serde_json::Value;

#[derive(Debug, PartialEq)]
pub enum Selector {
    ArrayIndex(usize),
    LastElementInArray,
    Key(String),
}

#[derive(Debug, PartialEq)]
pub enum JsonSelector {
    FullDocument,
    JsonSelector(Vec<Selector>),
}

fn parse(input: &str) -> IResult<&str, JsonSelector> {
    alt((
        parse_full_document,
        map(
            preceded(
                tag("/"),
                separated_list1(tag("/"), alt((parse_array_index, to_next_component))),
            ),
            JsonSelector::JsonSelector,
        ),
    ))(input)
}

fn parse_full_document(input: &str) -> IResult<&str, JsonSelector> {
    map(eof, |_| JsonSelector::FullDocument)(input)
}

fn to_next_component(input: &str) -> IResult<&str, Selector> {
    alt((
        map(take_till1(|c| c == '/'), |v: &str| {
            Selector::Key(v.replace("~1", "/").replace("~0", "~"))
        }),
        map(tag(""), |_| Selector::Key("".to_string())),
    ))(input)
}

fn parse_array_index(input: &str) -> IResult<&str, Selector> {
    alt((
        map(tag("-"), |_| Selector::LastElementInArray),
        map(parse_usize, Selector::ArrayIndex),
    ))(input)
}

fn parse_usize(input: &str) -> IResult<&str, usize> {
    map_res(recognize(digit1), str::parse)(input)
}

pub fn value_at(document: &Value, selector: &str) -> Result<Value, String> {
    let selector = parse(selector).unwrap().1;

    match selector {
        JsonSelector::FullDocument => Ok(document.clone()),
        JsonSelector::JsonSelector(mut values) => {
            value_at_selector(document, &mut values).ok_or("Unable to find".to_string())
        }
    }
}

pub fn mutate_at(document: &mut Value, selector: &str, new_value: Value) {
    let selector = parse(selector).unwrap().1;

    match selector {
        JsonSelector::FullDocument => {
            *document = new_value;
        }
        JsonSelector::JsonSelector(mut values) => {
            mutate_at_selector(document, &mut values, new_value)
        }
    }
}

fn mutate_at_selector(document: &mut Value, selectors: &mut Vec<Selector>, new_value: Value) {
    if selectors.is_empty() {
        *document = new_value;
    } else {
        let current_key = selectors.remove(0);
        match (document, current_key) {
            (Value::Array(array_value), Selector::ArrayIndex(idx)) => {
                if array_value.len() == idx && selectors.is_empty() {
                    array_value.push(new_value);
                } else {
                    if idx < array_value.len() {
                        mutate_at_selector(&mut array_value[idx], selectors, new_value)
                    }
                }
            }

            (Value::Array(array_value), Selector::LastElementInArray) => {
                if let Some(mut last) = array_value.last_mut() {
                    mutate_at_selector(&mut last, selectors, new_value)
                }
            }

            (Value::Object(ref mut dict), Selector::Key(ref key)) => {
                if let Some(mut value) = dict.get_mut(key) {
                    mutate_at_selector(&mut value, selectors, new_value)
                }
            }
            (_, _) => (),
        }
    }
}

fn value_at_selector(document: &Value, selectors: &mut Vec<Selector>) -> Option<Value> {
    if selectors.is_empty() {
        Some(document.clone())
    } else {
        let current_key = selectors.remove(0);
        match (document, current_key) {
            (Value::Array(array_value), Selector::ArrayIndex(idx)) => {
                value_at_selector(&array_value[idx], selectors)
            }

            (Value::Array(array_value), Selector::LastElementInArray) => {
                if let Some(last) = array_value.last() {
                    value_at_selector(&last, selectors)
                } else {
                    None
                }
            }

            (Value::Object(ref dict), Selector::Key(ref key)) => {
                if let Some(value) = dict.get(key) {
                    value_at_selector(&value, selectors)
                } else {
                    None
                }
            }
            (_, _) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn test_from_rfc6901() {
        let base_document = json(
            r#"
{
    "foo": ["bar", "baz"],
    "": 0,
    "a/b": 1,
    "c%d": 2,
    "e^f": 3,
    "g|h": 4,
    "i\\j": 5,
    "k\"l": 6,
    " ": 7,
    "m~n": 8
}
        "#,
        );
        assert_eq!(value_at(&base_document, ""), Ok(base_document.clone()));
        assert_eq!(
            value_at(&base_document, "/foo",),
            Ok(json("[\"bar\", \"baz\"]"))
        );
        assert_eq!(value_at(&base_document, "/foo/0",), Ok(json("\"bar\"")));
        assert_eq!(value_at(&base_document, "/",), Ok(json("0")));
        assert_eq!(value_at(&base_document, "/a~1b"), Ok(json("1")));
        assert_eq!(value_at(&base_document, "/c%d"), Ok(json("2")));
        assert_eq!(value_at(&base_document, "/e^f"), Ok(json("3")));
        assert_eq!(value_at(&base_document, "/g|h"), Ok(json("4")));
        assert_eq!(value_at(&base_document, "/i\\j"), Ok(json("5")));
        assert_eq!(value_at(&base_document, "/k\"l"), Ok(json("6")));
        assert_eq!(value_at(&base_document, "/ "), Ok(json("7")));
        assert_eq!(value_at(&base_document, "/m~0n"), Ok(json("8")));
    }

    #[test]
    fn parse_full_document_works() {
        assert_eq!(parse("").unwrap().1, JsonSelector::FullDocument);
    }

    #[test]
    fn parse_tilde() {
        assert_eq!(
            parse("/m~0n").unwrap().1,
            JsonSelector::JsonSelector(vec![Selector::Key("m~n".to_string())])
        );
    }

    #[test]
    fn parse_empty_key() {
        assert_eq!(
            parse("/").unwrap().1,
            JsonSelector::JsonSelector(vec![Selector::Key("".to_string())])
        );
    }

    #[test]
    fn parse_slash() {
        assert_eq!(
            parse("/m~1n").unwrap().1,
            JsonSelector::JsonSelector(vec![Selector::Key("m/n".to_string())])
        );
    }

    #[test]
    fn parse_array_selector_works() {
        assert_eq!(
            parse("/a/0").unwrap().1,
            JsonSelector::JsonSelector(vec![
                Selector::Key("a".to_string()),
                Selector::ArrayIndex(0)
            ])
        );
    }

    #[test]
    fn mutate_at_selector() {
        let mut base = json("{\"a\": 1}");
        mutate_at(&mut base, "/a", json("[2, 1]"));
        mutate_at(&mut base, "/a/1", json("3"));
        mutate_at(&mut base, "/a/2", json("0"));
        mutate_at(&mut base, "/a/4", json("\"nope\""));
        mutate_at(&mut base, "/a/3", json("{\"new-object\": [1,2]}"));
        mutate_at(&mut base, "/a/3/new-object/0", json("3"));
        mutate_at(&mut base, "/a/5/new-object/0", json("3"));
        assert_eq!(base, json("{\"a\": [2, 3, 0, {\"new-object\": [3,2]}]}"));
    }

    fn json(input: &str) -> Value {
        serde_json::from_str(input).unwrap()
    }
}
