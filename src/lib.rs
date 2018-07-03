extern crate failure;
extern crate i3ipc;
#[macro_use]
extern crate nom;
extern crate unicode_segmentation;

use failure::Error;
use failure::err_msg;

use i3ipc::reply::Workspaces;
use i3ipc::reply::Workspace;
use i3ipc::I3Connection;

use nom::{digit, rest};


pub trait I3ConnectionExt {
    fn run_commands(&mut self, cmds: &[String]) -> Result<(), Error>;
}

impl I3ConnectionExt for I3Connection {
    fn run_commands(&mut self, cmds: &[String]) -> Result<(), Error> {
        cmds.iter().for_each(|c| println!("`{}`", c));
        self.run_command(&cmds.join("; "))?;
        Ok(())
    }
}


pub trait WorkspacesExt {
    fn fixup_wps(&self) -> Vec<String>;
    fn choices_str(&self) -> String;

    fn get_wp_with_focus(&self) -> Option<&Workspace>;
    fn get_wp_with_name(&self, name: &str) -> Option<&Workspace>;
    fn get_wp_with_number(&self, number: usize) -> Option<&Workspace>;
    fn get_wp_with_title(&self, title: &str) -> Option<&Workspace>;

    fn go_to(&self, name: &str) -> Result<String, Error>;
    fn send_to(&self, name: &str) -> Result<String, Error>;
}

impl WorkspacesExt for Workspaces {
    fn fixup_wps(&self) -> Vec<String> {
        let mut cmds: Vec<String> = vec![];
        for (i, wp) in self.workspaces.iter().enumerate() {
            let old_num = if wp.num >= 0 { wp.num as usize } else { 0 };
            let new_num = i + 1;
            if old_num != new_num {
                cmds.push(format!(
                    "rename workspace \"{}\" to \"{}:{}\"",
                    wp.name,
                    new_num,
                    wp.title()
                ));
            }
        }
        cmds
    }

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

    fn get_wp_with_focus(&self) -> Option<&Workspace> {
        self.workspaces.iter().find(|wp| wp.focused == true)
    }

    fn get_wp_with_name(&self, name: &str) -> Option<&Workspace> {
        self.workspaces.iter().find(|wp| wp.name == name)
    }

    fn get_wp_with_number(&self, number: usize) -> Option<&Workspace> {
        self.workspaces.iter().find(|wp| wp.num == number as i32)
    }

    fn get_wp_with_title(&self, title: &str) -> Option<&Workspace> {
        self.workspaces.iter().find(|wp| wp.title() == title)
    }

    fn go_to(&self, name: &str) -> Result<String, Error> {
        let command = match self.get_wp_with_name(&name) {
            Some(_) => format!("workspace \"{}\"", name),
            None => format!("workspace \"{}\"", title_from_name(name)?),
        };
        Ok(command)
    }

    fn send_to(&self, name: &str) -> Result<String, Error> {
        let command = match self.get_wp_with_name(&name) {
            Some(_) => format!("move container to workspace \"{}\"", name),
            None => format!("move container to workspace \"{}\"", title_from_name(name)?),
        };
        Ok(command)
    }
}


pub trait WorkspaceExt {
    fn promote(&self) -> String;
    fn title<'a>(&'a self) -> &'a str;
}

impl WorkspaceExt for Workspace {
    fn promote(&self) -> String {
        format!("rename workspace \"{}\" to \"0:{}\"", self.name, self.title())
    }

    fn title<'a>(&'a self) -> &'a str {
        title_from_name(&self.name).unwrap()
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

fn title_from_name<'a>(name: &'a str) -> Result<&'a str, Error> {
    match parse_title_from_name(name.as_bytes()).to_full_result() {
        Ok(title) => Ok(title),
        Err(e) => Err(err_msg(
            format!("Couldn't parse title from name {:?}: {:?}", name, e),
        )),
    }
}
