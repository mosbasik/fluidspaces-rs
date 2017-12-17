extern crate failure;
extern crate i3ipc;
#[macro_use]
extern crate nom;
extern crate unicode_segmentation;

use failure::Error;

use i3ipc::reply::Workspaces;
use i3ipc::reply::Workspace;
use i3ipc::I3Connection;
// use i3ipc::MessageError;

use nom::{digit, rest};


pub trait I3ConnectionExt {
    fn run_commands(&mut self, cmds: &[String]) -> Result<(), Error>;

    fn promote_wp(&mut self, name: &str, title: &str) -> Result<(), Error>;
    fn wp_with_name_exists(&mut self, name: &str) -> Result<bool, Error>;

    fn name_from_title(&mut self, title: &str) -> Result<String, Error>;

    fn fixup_wps(&mut self) -> Result<(), Error>;

    fn go_to(&mut self, title: &str) -> Result<Vec<String>, Error>;
    fn send_to(&mut self, title: &str) -> Result<Vec<String>, Error>;
    fn bring_to(&mut self, title: &str) -> Result<Vec<String>, Error>;
}

impl I3ConnectionExt for I3Connection {
    fn run_cmds(&mut self, cmds: &[String]) -> Result<(), Error> {
        self.run_command(&cmds.join("; "))?;
        Ok(())
    }

    fn promote_wp(&mut self, name: &str, title: &str) -> Result<(), Error> {
        self.run_command(
            &format!("rename workspace {} to 0:{}", name, title),
        )?;
        self.fixup_wps()?;
        Ok(())
    }

    fn name_from_title(&mut self, title: &str) -> Result<String, Error> {
        let name = self.get_workspaces()?
            .get_wp_with_title(title)
            .ok_or(failure::err_msg(
                format!("No workspace with title \"{}\" found", title),
            ))?
            .name
            .clone();
        Ok(name)
    }

    fn wp_with_name_exists(&mut self, name: &str) -> Result<bool, Error> {
        match self.get_workspaces()?.get_wp_with_name(name) {
            Some(_) => Ok(true),
            None => Ok(false),
        }
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
        command_vector.iter().for_each(|c| println!("`{}`", c));
        self.run_command(&command_string)?;
        Ok(())
    }

    fn go_to(&mut self, name: &str) -> Result<Vec<String>, Error> {
        let target = match self.wp_with_name_exists(name)? {
            true => name,
            false => title_from_name(name),
        };
        Ok(vec![format!("workspace {}", target)])
    }

    fn send_to(&mut self, name: &str) -> Result<Vec<String>, Error> {
        let target = match self.wp_with_name_exists(name)? {
            true => name,
            false => title_from_name(name),
        };
        Ok(vec![format!("move container to workspace {}", target)])
    }

    fn bring_to(&mut self, name: &str) -> Result<Vec<String>, Error> {
        let mut result = vec![];
        result.extend(self.send_to(name)?.into_iter());
        result.extend(self.go_to(name)?.into_iter());
        Ok(result)
    }
}


pub trait WorkspacesExt {
    fn choices_str(&self) -> String;
    fn get_wp_with_name(&self, name: &str) -> Option<&Workspace>;
    fn get_wp_with_number(&self, number: usize) -> Option<&Workspace>;
    fn get_wp_with_title(&self, title: &str) -> Option<&Workspace>;
}

impl WorkspacesExt for Workspaces {
    fn choices_str(&self) -> String {
        let mut numbered_titles: Vec<(usize, &str)> = self.workspaces
            .iter()
            .map(|wp| (wp.num as usize, wp.title()))
            .collect();
        numbered_titles.sort_unstable_by(|a, b| (&a.0).cmp(&b.0));
        numbered_titles
            .iter()
            .map(|t| &*t.1)
            .collect::<Vec<&str>>()
            .join("\n")
    }

    fn get_wp_with_name(&self, name: &str) -> Option<&Workspace> {
        self.workspaces.iter().find(|wp| wp.name == name)
    }

    fn get_wp_with_title(&self, title: &str) -> Option<&Workspace> {
        self.workspaces.iter().find(|wp| wp.title() == title)
    }

    fn get_wp_with_number(&self, number: usize) -> Option<&Workspace> {
        self.workspaces.iter().find(|wp| wp.num == number as i32)
    }
}


pub trait WorkspaceExt {
    fn title<'a>(&'a self) -> &'a str;
}

impl WorkspaceExt for Workspace {
    fn title<'a>(&'a self) -> &'a str {
        title_from_name(&self.name)
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
        opt!(ws!(usize_parser)) >>
        opt!(ws!(tag!(":"))) >>
        title: ws!(title_parser) >>
        (title)
    )
));

fn title_from_name<'a>(name: &'a str) -> &'a str {
    parse_title_from_name(name.as_bytes()).to_result().unwrap()
}
