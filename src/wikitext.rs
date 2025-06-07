use std::borrow::Cow;

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::{
        complete::{take_until, take_while},
        tag,
    },
    combinator::map,
    sequence::delimited,
};

pub enum Value {
    Text(String),
    Hr,
}

/// Wikitext parser
// pub fn parse(text: &str) -> Vec<Value> {}

fn parse_block(input: &str) -> IResult<&str, Value> {
    alt([parse_hr]).parse(input)
}

fn parse_hr(input: &str) -> IResult<&str, Value> {
    let (i, _) = tag("----").parse(input)?;
    let (i, _) = take_while(|c| c == '-').parse(i)?;
    Ok((i, Value::Hr))
}

fn parse_text(input: &str) -> IResult<&str, Value> {
    map(take_until("{{"), |t: &str| Value::Text(t.to_string())).parse(input)
}

// fn parse_link(input: &str) -> IResult<&str, Value> {}

fn parse_template(input: &str) -> IResult<&str, Value> {
    delimited(tag("{{"), parse_text, tag("}}")).parse(input)
}
