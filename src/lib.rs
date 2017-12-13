extern crate failure;
extern crate i3ipc;
#[macro_use]
extern crate nom;
extern crate unicode_segmentation;

use failure::Error;

use i3ipc::reply::Workspaces;
use i3ipc::reply::Workspace;
use i3ipc::I3Connection;
use i3ipc::MessageError;

use nom::{digit, rest};

// use std::error::Error;


pub trait I3ConnectionExt {
    fn promote_wp_title(&mut self, title: &str) -> Result<(), Error>;
    fn fixup_wps(&mut self) -> Result<(), Error>;

    fn go_to(&mut self, title: &str) -> Result<(), Error>;
    fn send_to(&mut self, title: &str) -> Result<(), Error>;
    // fn bring_to(&mut self, title: &str) -> Result<(), Error>;
}

impl I3ConnectionExt for I3Connection {
    fn promote_wp_title(&mut self, title: &str) -> Result<(), Error> {
        let wp_name = self.get_workspaces()?
            .get_wp_with_title(title)
            .ok_or(failure::err_msg(
                format!("No workspace with title \"{}\" found", title),
            ))?
            .name
            .clone();
        let command_string = format!("rename workspace {} to 0:{}", wp_name, title);
        println!("`{}`", &command_string);
        self.run_command(&command_string)?;
        self.fixup_wps()?;
        Ok(())
    }

    fn fixup_wps(&mut self) -> Result<(), Error> {

        // get the the current list of workspaces from i3
        let wps = self.get_workspaces()?.workspaces;

        // initialize a vector to accumulate all of the commands the fixup requires, so they can be
        // joined together at the end and sent to i3 as a single operation
        let mut command_vector: Vec<String> = Vec::new();

        for (i, wp) in wps.iter().enumerate() {

            // the old number is what this workspace was numbered before the fixup
            let old_num = match wp.num {
                n if n >= 0 => n as usize,
                // _ => {
                //     return Err(failure::err_msg(
                //         format!("Unsupported: `{}` has negative number", wp.name),
                //     ))
                // }
                _ => 0,
            };

            // the new number is what this workspace will be numbered after the fixup
            let new_num = i + 1;

            // if the old and new numbers don't match, add a "rename" command to the vector
            if old_num != new_num {
                command_vector.push(format!(
                    "rename workspace {} to {}:{}",
                    wp.name,
                    new_num,
                    wp.title()
                ));
            }
        }

        // join the command vector into a single command string for i3 to execute (atomically?)
        let command_string = command_vector.join("; ");

        // send the command string to i3 to be executed
        // println!("running renames...", );
        command_vector.iter().for_each(|c| println!("`{}`", c));
        self.run_command(&command_string)?;
        Ok(())
    }

    fn go_to(&mut self, title: &str) -> Result<(), Error> {
        let (number, is_preexisting_wp) = match self.promote_wp_title(title) {
            Ok(_) => (1, true),
            Err(_) => (0, false),
        };
        self.run_command(&format!("workspace {}:{}", number, title))?;
        if !is_preexisting_wp {
            self.fixup_wps()?;
        }
        Ok(())
    }

    fn send_to(&mut self, title: &str) -> Result<(), Error> {
        let wps = self.get_workspaces()?;
        let (name, is_preexisting_wp) = match wps.get_wp_with_title(title) {
            Some(wp) => (format!("{}:{}", wp.num, title), true),
            None => (format!("{}", title), false),
        };



        // let foo = command_string;
        // println!("{:?}", foo);

        self.run_command(
            &format!("move container to workspace {}", name),
        )?;
        if !is_preexisting_wp {
            self.fixup_wps()?;
        }
        Ok(())

        // Err(failure::err_msg("implement I3ConnectionExt::send_to"))
    }

    // fn bring_to(&mut self, title: &str) -> Result<(), String> {
    //     Err(String::from("implement I3ConnectionExt::bring_to"))
    // }
}


pub trait WorkspaceExt {
    fn title(&self) -> String;
}

impl WorkspaceExt for Workspace {
    fn title(&self) -> String {
        parse_title_from_name(self.name.as_bytes())
            .to_result()
            .unwrap()
            .to_owned()
    }
}


pub trait WorkspacesExt {
    fn choices_str(&self) -> String;
    fn get_wp_with_title(&self, title: &str) -> Option<&Workspace>;
}

impl WorkspacesExt for Workspaces {
    fn choices_str(&self) -> String {
        let mut numbered_titles: Vec<(usize, String)> = self.workspaces
            .iter()
            .map(|wp| (wp.num as usize, wp.title()))
            .collect();
        numbered_titles.sort_unstable_by(|a, b| (&a.0).cmp(&b.0));
        numbered_titles
            .iter()
            .map(|t| &*t.1)
            .collect::<Vec<&str>>()
            .join(" ")
    }

    fn get_wp_with_title(&self, title: &str) -> Option<&Workspace> {
        self.workspaces.iter().find(|wp| wp.title() == title)
    }
}


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
        // opt!(take_while!(not!(nom::is_digit))) >>
        // not!(ws!(usize_parser)) >>
        opt!(ws!(usize_parser)) >>
        opt!(ws!(tag!(":"))) >>
        title: ws!(title_parser) >>
        (title)
    )
));
