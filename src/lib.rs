extern crate i3ipc;
#[macro_use]
extern crate nom;
extern crate unicode_segmentation;

use i3ipc::reply::Workspaces;
use i3ipc::reply::Workspace;
use i3ipc::I3Connection;

use nom::{digit, rest};

// use std::error::Error;


pub trait I3ConnectionExt {
    fn promote_wp_title(&mut self, title: &str) -> Result<(), String>;
    fn fixup_wps(&mut self) -> Result<(), String>;

    fn go_to(&self, title: &str) -> Result<(), String>;
    fn send_to(&self, title: &str) -> Result<(), String>;
    fn bring_to(&self, title: &str) -> Result<(), String>;
}

impl I3ConnectionExt for I3Connection {
    fn promote_wp_title(&mut self, title: &str) -> Result<(), String> {

        let wps = match self.get_workspaces() {
            Ok(workspaces) => workspaces.workspaces,
            Err(_) => {
                return Err(String::from(
                    "Running get_workspaces() caused i3ipc to return an error",
                ))
            }
        };

        let wp = match wps.into_iter().find(|wp| wp.title() == title) {
            Some(workspace) => workspace,
            None => return Err(format!("No workspace with title \"{}\" found", title)),
        };

        let command_string = format!("rename workspace {} to 0:{}", wp.name, title);

        match self.run_command(&command_string) {
            Ok(_) => self.fixup_wps(),
            Err(_) => Err(format!(
                "Running the command `{}` caused i3ipc to return an error",
                command_string
            )),
        }
    }

    fn fixup_wps(&mut self) -> Result<(), String> {

        // get the the current list of workspaces from i3
        let wps = match self.get_workspaces() {
            Ok(workspaces) => workspaces.workspaces,
            Err(_) => {
                return Err(String::from(
                    "Running get_workspaces() caused i3ipc to return an error",
                ))
            }
        };

        // initialize a vector to accumulate all of the commands the fixup requires, so they can be
        // joined together at the end and sent to i3 as a single operation
        let mut command_vector: Vec<String> = Vec::new();

        for (i, wp) in wps.iter().enumerate() {

            // the old number is what this workspace was numbered before the fixup
            let old_num = match wp.num {
                n if n >= 0 => n as usize,
                _ => return Err(format!("Unsupported: `{}` has negative number", wp.name)),
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

        println!("{:?}", command_vector.join("; "));

        // send the command string to i3 to be executed
        match self.run_command(&command_string) {
            Ok(_) => Ok(()),
            Err(_) => Err(format!(
                "Running the command `{}` caused i3ipc to return an error",
                command_string
            )),
        }
    }

    fn go_to(&self, title: &str) -> Result<(), String> {
        Err(String::from("implement I3ConnectionExt::go_to"))
    }

    fn send_to(&self, title: &str) -> Result<(), String> {
        Err(String::from("implement I3ConnectionExt::send_to"))
    }

    fn bring_to(&self, title: &str) -> Result<(), String> {
        Err(String::from("implement I3ConnectionExt::bring_to"))
    }
}


pub trait WorkspaceExt {
    fn title(&self) -> String;
}

impl WorkspaceExt for Workspace {
    fn title(&self) -> String {
        let (_, title) = name_parser(self.name.as_bytes()).to_result().unwrap();
        String::from(title)
    }
}


pub trait WorkspacesExt {
    fn choices_str(&self) -> String;
    fn gapless_rename_lists(&self) -> (Vec<String>, Vec<String>);
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

    fn gapless_rename_lists(&self) -> (Vec<String>, Vec<String>) {
        let mut old_i3_names: Vec<String> = Vec::new();
        let mut new_i3_names: Vec<String> = Vec::new();
        for (gapless_num, wp) in self.workspaces.iter().enumerate() {
            let gapless_i3_name = format!("{}:{}", gapless_num + 1, wp.title());
            if gapless_i3_name != wp.name {
                old_i3_names.push(wp.name.clone());
                new_i3_names.push(gapless_i3_name.clone());
            }
        }
        (old_i3_names, new_i3_names)
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

named!(name_parser<(usize, &str)>,
    do_parse!(
        number: ws!(usize_parser) >>
        ws!(tag!(":")) >>
        title: ws!(title_parser) >>
        (number, title)
    )
);
