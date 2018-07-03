extern crate failure;

use failure::Error;
use failure::err_msg;

use nom::{digit, rest};

use std;


named!(usize_parser<usize>,
    map_res!(
        map_res!(
            digit,
            std::str::from_utf8
        ),
        std::str::FromStr::from_str
    )
);

named!(title_parser<&str>,
    map_res!(
        rest,
        std::str::from_utf8
    )
);

named!(pub parse_title_from_name<&str>, dbg!(
    do_parse!(
        opt!(ws!(usize_parser)) >>
        opt!(ws!(tag!(":"))) >>
        title: ws!(title_parser) >>
        (title)
    )
));

pub fn title_from_name<'a>(name: &'a str) -> Result<&'a str, Error> {
    match parse_title_from_name(name.as_bytes()).to_full_result() {
        Ok(title) => Ok(title),
        Err(e) => Err(err_msg(
            format!("Couldn't parse title from name {:?}: {:?}", name, e),
        )),
    }
}


#[cfg(test)]
mod tests {
    // use nom::IResult;
    // use super::usize_parser;
    use super::title_from_name;

    // #[test]
    // fn usize_parser_single_digit() {
    //     assert_eq!(usize_parser(&b"7"[..]), IResult::Done(&b""[..], 7));
    // }

    // define a macro that can generate test functions for us
    macro_rules! title_from_name_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (input, expected) = $value;
                    assert_eq!(title_from_name(input).unwrap(), expected);
                }
            )*
        };
    }

    title_from_name_tests! {
        basic: ("7:foo", "foo"),

        number_is_multidigit: ("77:foo", "foo"),
        number_is_missing: (":foo", "foo"),
        number_and_colon_are_missing: ("foo", "foo"),

        space_after_colon: ("7: foo", "foo"),
        space_before_colon: ("7 :foo", "foo"),
        space_around_colon: ("7 : foo", "foo"),

        title_has_space: ("7:foo bar", "foo bar"),
        title_has_hyphen: ("7:foo-bar", "foo-bar"),
        title_has_apostrophe: ("7:foo'bar", "foo'bar"),

        title_has_number_inside: ("7:fo2o", "fo2o"),
        title_has_number_after: ("7:foo2", "foo2"),
        title_has_number_before: ("7:2foo", "2foo"),
        title_has_numberspace_before: ("7:2 foo", "2 foo"),
    }
}
