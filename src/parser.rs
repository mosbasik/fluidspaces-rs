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
