extern crate failure;
extern crate i3ipc;
extern crate clap;
extern crate fluidspaces;

// use clap::{Arg, ArgGroup, App};

use failure::Error;
use failure::err_msg;

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

use fluidspaces::WorkspaceExt;
use fluidspaces::WorkspacesExt;
use fluidspaces::I3ConnectionExt;

// use fluidspaces::parse_title_from_name;


fn main() {

    // establish connection with i3 IPC socket
    let mut i3 = I3Connection::connect().unwrap();

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

    // DEBUG
    println!("message: {:?}", &message);

    // make sure the message is one of the ones we expect before going any further
    match message.as_str() {
        "go_to" | "send_to" | "bring_to" | "toggle" => (),
        _ => {
            return Err(err_msg(
                format!("Unexpected message received: {:?}", message),
            ))
        }
    }

    // get Workspaces object from i3
    let workspaces = i3.get_workspaces()?;

    // establish the target workspace name (or title) for this action
    let target = match message.as_str() {

        // if the action is "toggle", that's all we need to find the target
        "toggle" => {
            match workspaces.get_wp_with_number(2) {
                Some(wp) => wp.name.clone(),
                None => return Err(err_msg(format!("Couldn't find workspace number 2"))),
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

                // write the list of workspaces to dmenu's stdin
                stdin.write_all(workspaces.choices_str().as_bytes())?;
            }

            // wait for dmenu to exit and collect all output on its stdout / stderr
            let menu_output = menu_proc.wait_with_output()?;

            // get the title chosen by the user from dmenu's stdout
            let raw_title = String::from_utf8_lossy(&menu_output.stdout[..]);
            let title = raw_title.trim();

            // the target is either the full workspace name (if a workspace exists with
            // a name that matches the chosen title) or the chosen title itself (if a
            // workspace with a matching name doesn't exist)
            match workspaces.get_wp_with_title(title) {
                Some(wp) => wp.name.clone(),
                None => title.to_owned(),
            }
        }
    };

    // DEBUG
    println!("target: {:?}", target);

    // initialize empty vector of action commands
    let mut action_cmds: Vec<String> = vec![];

    // push command strings into the vector according to the requested action
    match message.as_str() {
        "go_to" | "toggle" => action_cmds.push(workspaces.go_to(&target)?),
        "send_to" => action_cmds.push(workspaces.send_to(&target)?),
        "bring_to" => {
            action_cmds.push(workspaces.send_to(&target)?);
            action_cmds.push(workspaces.go_to(&target)?);
        }
        _ => panic!(),  // shouldn't happen
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
