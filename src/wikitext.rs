use std::borrow::Cow;

use nom::{IResult, Parser, bytes::complete::take_until, combinator::map, sequence::delimited};

pub enum Value<'a> {
    Text(Cow<'a, str>),
}

/// Wikitext parser
pub fn parse(text: &str) -> Vec<Value> {
    parse_text(text)
        .map(|t| vec![t.1])
        .expect("this parser is bad, lower expectations")
    // todo!("Implement");
}

fn parse_text(input: &str) -> IResult<&str, Value> {
    map(take_until("{{"), |t: &str| Value::Text(Cow::from(t))).parse(input)
}

// fn parse_template(input: &str) -> IResult<&str, Value> {
//     map(
//         delimited(tag("{{"), parse_text(), tag("}}")),
//         |s| parse_
//     ).parse(input)
// }
