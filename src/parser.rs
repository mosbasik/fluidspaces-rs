extern crate failure;

// use failure::err_msg;
use failure::Error;

use nom::types::CompleteByteSlice as Input;
use nom::{is_digit, rest};

// use std;

// // defines "usize_parser"
// // returns a usize parsed from the input string?
// named!(
//     usize_parser<usize>,
//     map_res!(
//         map_res!(digit, std::str::from_utf8),
//         std::str::FromStr::from_str
//     )
// );

// parser matches as many digits as possible and then stops; outputs the digits
// or an empty string
named!(number<Input, Input>, take_while!(is_digit));

// parser matches a colon if present and errors if not
named!(colon<Input, Input>, tag!(":"));

// parser matches all input until the end; outputs everything matched
named!(the_rest<Input, Input>, call!(rest));

// // defines "title_parser"
// // returns the rest of the input
// named!(title_parser<&str>, map_res!(rest, std::str::from_utf8));

// // defines "parse_title_from_name"
// // in order:
// //   parses and discards a usize (optional)
// //   parses and discards a colon (optional)
// //   returns the rest of the input
// named!(pub parse_title_from_name<&str>, dbg!(
//     do_parse!(
//         opt!(ws!(usize_parser)) >>
//         opt!(ws!(tag!(":"))) >>
//         title: ws!(title_parser) >>
//         (title)
//     )
// ));

pub fn title_from_name<'a>(name: &'a str) -> Result<&'a str, Error> {
    unimplemented!();
    // match parse_title_from_name(name.as_bytes()).to_full_result() {
    //     Ok(title) => Ok(title),
    //     Err(e) => Err(err_msg(format!(
    //         "Couldn't parse title from name {:?}: {:?}",
    //         name, e
    //     ))),
    // }
}

#[cfg(test)]
mod tests {
    use nom::{types::CompleteByteSlice as Input, Context::Code, Err::Error, ErrorKind};
    use super::{colon, number, the_rest};

    // define macro that will generate tests for any parser that uses Input type
    // for both input and output
    macro_rules! test_parser {(
        $parser:ident,
        $($name:ident: $value:expr,)*
    ) => {$(
        #[test]
        fn $name() {
            let (input, expected_result): (&str, nom::IResult<Input, Input>) = $value;
            match $parser(Input(input.as_bytes())) {
                Ok(actual_result) => assert_eq!(
                    actual_result,
                    expected_result.expect("Parser gave an Ok() but test expects an Err()")),
                Err(actual_result) => assert_eq!(
                    actual_result,
                    expected_result.expect_err("Parser gave an Err() but test expects an Ok()"))
            }
        }
    )*}}

    mod number_tests {
        use super::*;
        test_parser! {
            number,
            basic: ("7", Ok((Input(b""), Input(b"7")))),
            long_start: ("77", Ok((Input(b""), Input(b"77")))),
            trailing_letter: ("7a", Ok((Input(b"a"), Input(b"7")))),
            long_start_and_trailing_letter: ("77a", Ok((Input(b"a"), Input(b"77")))),
            immediate_letter: ("a", Ok((Input(b"a"), Input(b"")))),
            immediate_colon: (":", Ok((Input(b":"), Input(b"")))),
            number_after_letters: ("a7", Ok((Input(b"a7"), Input(b"")))),
            number_after_colon: (":7", Ok((Input(b":7"), Input(b"")))),
        }
    }

    mod colon_tests {
        use super::*;
        test_parser! {
            colon,
            basic: (":", Ok((Input(b""), Input(b":")))),
            colon_then_number: (":7", Ok((Input(b"7"), Input(b":")))),
            colon_then_letter: (":a", Ok((Input(b"a"), Input(b":")))),
            number_then_colon: ("7:", Err(Error(Code(Input(b"7:"), ErrorKind::Tag)))),
            letter_then_colon: ("a:", Err(Error(Code(Input(b"a:"), ErrorKind::Tag)))),
        }
    }

    mod the_rest_tests {
        use super::*;
        test_parser! {
            the_rest,
            nothing: ("", Ok((Input(b""), Input(b"")))),
            one_letter: ("a", Ok((Input(b""), Input(b"a")))),
            multiple_letters: ("aa", Ok((Input(b""), Input(b"aa")))),
            spaced_letters: ("a a", Ok((Input(b""), Input(b"a a")))),
            contains_colon: ("a:a", Ok((Input(b""), Input(b"a:a")))),
        }
    }

    // // define a macro that can generate test functions for us
    // macro_rules! title_from_name_tests {
    //     ($($name:ident: $value:expr,)*) => {
    //         $(
    //             #[test]
    //             fn $name() {
    //                 let (input, expected) = $value;
    //                 assert_eq!(title_from_name(input).unwrap(), expected);
    //             }
    //         )*
    //     };
    // }

    // title_from_name_tests! {
    //     basic: ("7:foo", "foo"),

    //     number_is_multidigit: ("77:foo", "foo"),
    //     number_is_missing: (":foo", "foo"),
    //     number_and_colon_are_missing: ("foo", "foo"),

    //     space_after_colon: ("7: foo", "foo"),
    //     space_before_colon: ("7 :foo", "foo"),
    //     space_around_colon: ("7 : foo", "foo"),

    //     title_has_space: ("7:foo bar", "foo bar"),
    //     title_has_hyphen: ("7:foo-bar", "foo-bar"),
    //     title_has_apostrophe: ("7:foo'bar", "foo'bar"),

    //     title_has_number_inside: ("7:fo2o", "fo2o"),
    //     title_has_number_after: ("7:foo2", "foo2"),
    //     title_has_number_before: ("7:2foo", "2foo"),
    //     title_has_numberspace_before: ("7:2 foo", "2 foo"),

    //     number_title_with_present_index_present_colon: ("7:2", "2"),
    //     number_title_with_missing_index_present_colon: (":2", "2"),
    //     // number_title_with_missing_index_missing_colon: ("2", "2"),
    // }
}
