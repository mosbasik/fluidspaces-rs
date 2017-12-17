extern crate i3ipc;
extern crate clap;
extern crate fluidspaces;

use clap::{Arg, ArgGroup, App};
use i3ipc::I3Connection;
// use i3ipc::reply::{Workspace, Workspaces};

use std::io::Write;

use std::process::Command;
use std::process::Stdio;

use fluidspaces::WorkspaceExt;
use fluidspaces::WorkspacesExt;
use fluidspaces::I3ConnectionExt;

// use fluidspaces::parse_title_from_name;


fn main() {

    let mut connection = I3Connection::connect().unwrap();

    let workspaces = connection.get_workspaces().unwrap();

    // println!("{:?}", workspaces.choices_str());

    // println!("{:?}", workspaces.get_wp_with_title("rust"));
    // println!("{:?}", workspaces.get_wp_with_title("foo"));

    // println!("{:?}", connection.promote_wp_title("rust"));

    // println!("{:?}", connection.fixup_wps());

    // println!("{:?}", connection.go_to("rust"));
    // println!("{:?}", connection.go_to("foo"));

    // println!("{:?}", connection.send_to("rust"));
    // println!("{:?}", connection.send_to("foo"));

    // assert_eq!(parse_title_from_name("1:foo".as_bytes()).to_result().unwrap(), "foo");
    // assert_eq!(parse_title_from_name("1: foo".as_bytes()).to_result().unwrap(), "foo");
    // assert_eq!(parse_title_from_name("1 :foo".as_bytes()).to_result().unwrap(), "foo");
    // assert_eq!(parse_title_from_name("1 : foo".as_bytes()).to_result().unwrap(), "foo");
    // assert_eq!(parse_title_from_name("foo".as_bytes()).to_result().unwrap(), "foo");
    // assert_eq!(parse_title_from_name(":foo".as_bytes()).to_result().unwrap(), "foo");
    // // assert_eq!(parse_title_from_name("-1:foo".as_bytes()).to_result().unwrap(), "foo");

    // println!("{:?}", parse_title_from_name("-1:foo".as_bytes()));


    let matches = App::new("fluidspaces")
        .version("0.4.0")
        .author("Peter Henry <me@peterhenry.net>")
        .about("Navigator for i3wm \"named containers\". Create i3 workspaces with custom names on the fly, navigate between them based on their their name or position, and move containers between them.")
        .arg(Arg::with_name(&"bring_to")
            .short("-b")
            .long("--bring-to")
            .help("Bring focused container with you to workspace"))
        .arg(Arg::with_name(&"send_to")
            .short("-s")
            .long("--send-to")
            .help("Send focused container away to workspace"))
        .group(ArgGroup::with_name("action")
            .arg("bring_to")
            .arg("send_to"))
        .arg(Arg::with_name("toggle")
            .short("-t")
            .long("--toggle")
            .help("Skip menu & choose workspace 2 (default: false)"))
        .arg(Arg::with_name("order")
            .short("-o")
            .long("--order")
            .possible_values(&["default", "last-used"])
            .default_value("default")
            .help("Method used to determine workspace display order"))
        // .arg(Arg::with_name("menu")
        //     .short("-m")
        //     .long("--menu")
        //     .possible_values(&["dmenu", "rofi"])
        //     .default_value("dmenu")
        //     .help("Program used to render the menu"))
        .get_matches();

    let target = if matches.is_present("toggle") {
        workspaces.get_wp_with_number(2).unwrap().name.clone()
    } else {
        let mut menu_proc = Command::new("dmenu")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to execute process");
        {
            let stdin = menu_proc.stdin.as_mut().expect("failed to get stdin");
            stdin.write_all(workspaces.choices_str().as_bytes()).expect("failed to write to stdin");
        }
        let menu_output = menu_proc.wait_with_output().expect("failed to wait on process");
        let title = std::str::from_utf8(menu_output.stdout.as_slice()).expect("failed to parse stdout").trim();
        match workspaces.get_wp_with_title(title) {
            Some(wp) => wp.name.clone(),
            None => title.to_owned(),
        }
    };

    let mut action_cmds: Vec<String> = vec![];
    if matches.is_present("send_to") {
        action_cmds.push(workspaces.send_to(&target));
    } else if matches.is_present("bring_to") {
        action_cmds.push(workspaces.send_to(&target));
        action_cmds.push(workspaces.go_to(&target));
    } else {
        action_cmds.push(workspaces.go_to(&target));
    };
    connection.run_commands(&action_cmds).expect("running generated commands failed");

    if matches.value_of("order").unwrap() == "last-used" {
        let promote_cmds = match connection.get_workspaces().expect("failed to get workspaces").get_wp_with_focus() {
            Some(wp) => vec![wp.promote()],
            None => vec![],
        };
        connection.run_commands(&promote_cmds).expect("running generated commands failed");
    }

    let fixup_cmds = connection.get_workspaces().expect("failed to get workspaces").fixup_wps();
    connection.run_commands(&fixup_cmds).expect("running generated commands failed");
}
