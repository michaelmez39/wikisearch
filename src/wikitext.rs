#[allow(dead_code)]
/// Removes wikitext macros
/// e.g. stuff{{123}} => stuff
///
/// TODO: Might be completely unneeded,
/// remove if this is the case
pub fn clean_wikitext(text: &str) -> String {
    text.split("{{")
        .map(|c| c.split("}}"))
        .flatten()
        .step_by(2)
        .fold(String::new(), |mut acc, c| {
            acc.push_str(c);
            acc
        })
}

#[cfg(test)]
mod test {
    use super::clean_wikitext;

    #[test]
    fn passthrough() {
        let t = String::from("Hello, world!");
        assert_eq!(t, clean_wikitext(&t))
    }

    #[test]
    fn strips() {
        assert_eq!(
            "Hello, world!",
            clean_wikitext("Hello, {{some stuff}}world!")
        )
    }

    #[test]
    fn strips_start() {
        assert_eq!("Hello, world!", clean_wikitext("{{stuff}}Hello, world!"))
    }

    #[test]
    fn strips_end() {
        assert_eq!("Hello, world!", clean_wikitext("Hello, world!{{stuff}}"))
    }

    #[test]
    fn strips_boundary() {
        assert_eq!(
            "Hello, world!",
            clean_wikitext("{{stuff}}Hello, world!{{stuff}}")
        )
    }

    #[test]
    fn strips_many() {
        assert_eq!(
            "Hello, world!",
            clean_wikitext("Hello,{{stuff}} world!{{stuff}}")
        )
    }

    #[test]
    fn nested() {
        assert_eq!("cat", clean_wikitext("ca{{ hi {{bye}}}}t"))
    }
}

// use std::borrow::Cow;

// use nom::{
//     IResult, Parser,
//     branch::alt,
//     bytes::{
//         complete::{take_until, take_while},
//         tag,
//     },
//     combinator::map,
//     sequence::delimited,
// };

// pub enum Value {
//     Text(String),
//     Hr,
// }

// Wikitext parser
// pub fn parse(text: &str) -> Vec<Value> {}

// fn parse_block(input: &str) -> IResult<&str, Value> {
//     alt([parse_hr]).parse(input)
// }

// fn parse_hr(input: &str) -> IResult<&str, Value> {
//     let (i, _) = tag("----").parse(input)?;
//     let (i, _) = take_while(|c| c == '-').parse(i)?;
//     Ok((i, Value::Hr))
// }

// fn parse_text(input: &str) -> IResult<&str, Value> {
//     map(take_until("{{"), |t: &str| Value::Text(t.to_string())).parse(input)
// }

// // fn parse_link(input: &str) -> IResult<&str, Value> {}

// fn parse_template(input: &str) -> IResult<&str, Value> {
//     delimited(tag("{{"), parse_text, tag("}}")).parse(input)
// }
