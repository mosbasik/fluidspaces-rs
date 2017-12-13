extern crate i3ipc;
extern crate clap;
extern crate fluidspaces;

use clap::{Arg, ArgGroup, App};
use i3ipc::I3Connection;
// use i3ipc::reply::{Workspace, Workspaces};
use std::process::Command;

use fluidspaces::WorkspacesExt;
use fluidspaces::I3ConnectionExt;

use fluidspaces::parse_title_from_name;


fn main() {

    // establish a connection to i3 over a unix socket
    let mut connection = I3Connection::connect().unwrap();

    let workspaces = connection.get_workspaces().unwrap();

    // println!("{:?}", workspaces.choices_str());
    // println!("{:?}", workspaces.gapless_rename_lists());

    // println!("{:?}", workspaces.get_wp_with_title("rust"));
    // println!("{:?}", workspaces.get_wp_with_title("foo"));

    // println!("{:?}", connection.promote_wp_title("rust"));

    // println!("{:?}", connection.fixup_wps());

    // println!("{:?}", connection.go_to("rust"));
    // println!("{:?}", connection.go_to("foo"));

    // println!("{:?}", connection.send_to("rust"));
    println!("{:?}", connection.send_to("foo"));

    assert_eq!(parse_title_from_name("1:foo".as_bytes()).to_result().unwrap(), "foo");
    assert_eq!(parse_title_from_name("1: foo".as_bytes()).to_result().unwrap(), "foo");
    assert_eq!(parse_title_from_name("1 :foo".as_bytes()).to_result().unwrap(), "foo");
    assert_eq!(parse_title_from_name("1 : foo".as_bytes()).to_result().unwrap(), "foo");
    assert_eq!(parse_title_from_name("foo".as_bytes()).to_result().unwrap(), "foo");
    assert_eq!(parse_title_from_name(":foo".as_bytes()).to_result().unwrap(), "foo");
    // assert_eq!(parse_title_from_name("-1:foo".as_bytes()).to_result().unwrap(), "foo");

    // println!("{:?}", parse_title_from_name("-1:foo".as_bytes()));





    let _matches = App::new("fluidspaces")
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
        .arg(Arg::with_name("menu")
            .short("-m")
            .long("--menu")
            .possible_values(&["dmenu", "rofi"])
            .default_value("dmenu")
            .help("Program used to render the menu"))
        .get_matches();

    let _whoami = Command::new("whoami")
        .output()
        .expect("failed to execute process")
        .stdout;

}
