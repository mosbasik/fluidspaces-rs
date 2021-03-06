extern crate clap;
extern crate failure;
extern crate fluidspaces;
extern crate i3ipc;

// use clap::{Arg, ArgGroup, App};

use failure::err_msg;
use failure::Error;

use i3ipc::I3Connection;
// use i3ipc::reply::{Workspace, Workspaces};

use std::fs;

use std::io::Write;
// use std::io::ErrorKind;
// use std::io::Error;
use std::io::Read;

// use std::net::Shutdown;

use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
// use std::os::unix::net::UnixDatagram;

use std::process::Command;
use std::process::Stdio;

use fluidspaces::go_to;
use fluidspaces::send_to;
use fluidspaces::I3ConnectionExt;
use fluidspaces::WorkspaceExt;
use fluidspaces::WorkspacesExt;

// use fluidspaces::parse_title_from_name;

fn main() {
    // establish connection with i3 IPC socket
    let mut i3 = match I3Connection::connect() {
        Ok(connection) => connection,
        Err(e) => panic!("Couldn't connect to i3: {:?}", e),
    };

    // define filename for fluidspaces IPC socket
    let socket_filename = "/tmp/fluidspaces.sock";

    // try to delete old socket file; ignore success and failure
    match fs::remove_file(socket_filename) {
        _ => (),
    };

    // bind listener to a new socket
    let listener = match UnixListener::bind(socket_filename) {
        Ok(sock) => sock,
        Err(e) => panic!("Couldn't bind to new socket: {:?}", e),
    };

    // set listener behavior to "blocking"
    if let Err(e) = listener.set_nonblocking(false) {
        panic!("Couldn't set socket listener to blocking mode: {:?}", e);
    }

    // start event loop - blocks until the socket receives a message
    for stream_res in listener.incoming() {
        match stream_res {
            // if the stream was successfully read from the socket
            Ok(mut stream) => {
                println!("----------"); // DEBUG

                // process the stream
                if let Err(e) = handle_stream(&mut i3, &mut stream) {
                    eprintln!("{}", e.cause());
                }
            }
            // if the stream failed to be read from the socket
            Err(e) => eprintln!("Couldn't read stream from socket: {:?}", e),
        }
    }
}

fn handle_stream(i3: &mut I3Connection, stream: &mut UnixStream) -> Result<(), Error> {
    // decode the stream's contents as UTF8 and save it into the string "message"
    let mut message = String::new();
    stream.read_to_string(&mut message)?;

    println!("message: {:?}", &message); // DEBUG

    // get Workspaces object from i3
    let workspaces = i3.get_workspaces()?;

    // establish the target workspace name (or title) for this action
    let target = match message.as_str() {
        // if the action is "toggle"
        "toggle" => {
            // determine the currently focused workspace and get the name of the output it's on
            let active_output = match workspaces.workspaces.iter().find(|wp| wp.focused == true) {
                Some(wp) => wp.output.clone(),
                None => return Err(err_msg(format!("Couldn't find a focused workspace"))),
            };

            println!("active output: \"{}\"", active_output);

            // determine the name of the second workspace on the active output
            match workspaces.get_second_wp_with_output(&active_output) {
                Some(wp) => wp.name.clone(),
                None => {
                    return Err(err_msg(format!(
                        "Couldn't find a second workspace on the active output"
                    )))
                }
            }
        }

        // if the action isn't "toggle", we have to ask the user to specify a target
        _ => {
            // spawn a dmenu process
            let mut menu_proc = Command::new("dmenu")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()?;

            {
                // get a ref to stdin of dmenu process
                let stdin = match menu_proc.stdin.as_mut() {
                    Some(s) => s,
                    None => return Err(err_msg(format!("Couldn't get ref to stdin of dmenu"))),
                };

                // println!("workspace choices: {:?}", workspaces.choices_str());  // DEBUG

                // write the list of workspaces to dmenu's stdin
                stdin.write_all(workspaces.choices_str().as_bytes())?;
            }

            // wait for dmenu to exit and collect all output on its stdout / stderr
            let menu_output = menu_proc.wait_with_output()?;

            // get the title chosen by the user from dmenu's stdout
            let raw_title = String::from_utf8_lossy(&menu_output.stdout[..]);
            // println!("choice (raw title) {:?}", raw_title);  // DEBUG
            let title = raw_title.trim();
            // println!("choice (title) {:?}", title);  // DEBUG

            // check to see if the user actually chose a target; if they didn't then we don't need
            // to do anything else and can return early
            if title == "" {
                return Ok(());
            }

            // the target is either an existing workspace name (if a workspace
            // with a matching title exists) or the combination of the next
            // unused number and the chosen title itself (if a workspace with a
            // matching title doesn't exist)
            match workspaces.get_wp_with_title(title) {
                Some(wp) => wp.name.clone(),
                None => format!("{}:{}", workspaces.next_unused_number(), title),
            }
        }
    };

    println!("target: {:?}", target); // DEBUG

    // initialize empty vector of action commands
    let mut action_cmds: Vec<String> = vec![];

    // push command strings into the vector according to the requested action
    match message.as_str() {
        "go_to" | "toggle" => action_cmds.push(go_to(&target)),
        "send_to" => action_cmds.push(send_to(&target)),
        "bring_to" => {
            action_cmds.push(send_to(&target));
            action_cmds.push(go_to(&target));
        }
        message => {
            return Err(err_msg(format!(
                "Unexpected message received: {:?}",
                message
            )))
        }
    }

    // run action commands all at once
    i3.run_commands(&action_cmds)?;

    // initialize vector of promotion commands
    let promote_cmds = match i3.get_workspaces()?.get_wp_with_focus() {
        Some(wp) => vec![wp.promote()],
        None => vec![],
    };

    // run promotion commands all at once
    i3.run_commands(&promote_cmds)?;

    // initialize vector of fixup commands
    let fixup_cmds = i3.get_workspaces()?.fixup_wps();

    // run fixup commands all at once
    i3.run_commands(&fixup_cmds)?;

    Ok(())
}

// testing stuff for later

// println!("{:?}", workspaces.choices_str());

// println!("{:?}", workspaces.get_wp_with_title("rust"));
// println!("{:?}", workspaces.get_wp_with_title("foo"));

// println!("{:?}", i3.promote_wp_title("rust"));

// println!("{:?}", i3.fixup_wps());

// println!("{:?}", i3.go_to("rust"));
// println!("{:?}", i3.go_to("foo"));

// println!("{:?}", i3.send_to("rust"));
// println!("{:?}", i3.send_to("foo"));

// assert_eq!(parse_title_from_name("1:foo".as_bytes()).to_result().unwrap(), "foo");
// assert_eq!(parse_title_from_name("1: foo".as_bytes()).to_result().unwrap(), "foo");
// assert_eq!(parse_title_from_name("1 :foo".as_bytes()).to_result().unwrap(), "foo");
// assert_eq!(parse_title_from_name("1 : foo".as_bytes()).to_result().unwrap(), "foo");
// assert_eq!(parse_title_from_name("foo".as_bytes()).to_result().unwrap(), "foo");
// assert_eq!(parse_title_from_name(":foo".as_bytes()).to_result().unwrap(), "foo");
// // assert_eq!(parse_title_from_name("-1:foo".as_bytes()).to_result().unwrap(), "foo");

// println!("{:?}", parse_title_from_name("-1:foo".as_bytes()));
