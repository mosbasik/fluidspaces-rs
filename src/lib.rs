extern crate failure;
extern crate i3ipc;
#[macro_use]
extern crate nom;
extern crate unicode_segmentation;
mod parser;

use failure::Error;

use i3ipc::reply::Workspace;
use i3ipc::reply::Workspaces;
use i3ipc::I3Connection;

use parser::title_from_name;

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

    fn get_second_wp_with_output(&self, output: &str) -> Option<&Workspace>;

    fn next_unused_number(&self) -> usize;
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
        let mut numbered_titles: Vec<(usize, &str)> = self
            .workspaces
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

    fn next_unused_number(&self) -> usize {
        self.workspaces.len() + 1
    }

    fn get_second_wp_with_output(&self, output: &str) -> Option<&Workspace> {
        self.workspaces
            .iter()
            .filter(|wp| wp.output == output)
            .nth(1)
    }
}

pub trait WorkspaceExt {
    fn promote(&self) -> String;
    fn title<'a>(&'a self) -> &'a str;
}

impl WorkspaceExt for Workspace {
    fn promote(&self) -> String {
        format!(
            "rename workspace \"{}\" to \"0:{}\"",
            self.name,
            self.title()
        )
    }

    fn title<'a>(&'a self) -> &'a str {
        title_from_name(&self.name).unwrap()
    }
}

pub fn go_to(name: &str) -> String {
    format!("workspace \"{}\"", name)
}

pub fn send_to(name: &str) -> String {
    format!("move container to workspace \"{}\"", name)
}
