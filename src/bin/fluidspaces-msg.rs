extern crate clap;

use clap::App;
use clap::Arg;

use std::io::Write;

use std::os::unix::net::UnixStream;


fn main() {
    let matches = App::new("fluidspaces")
        .version("0.4.0")
        .author("Peter Henry <me@peterhenry.net>")
        .about("Navigator for i3wm \"named containers\". Create i3 workspaces with custom names on the fly, navigate between them based on their their name or position, and move containers between them.")
        // .arg(Arg::with_name(&"bring_to")
        //     .short("-b")
        //     .long("--bring-to")
        //     .help("Bring focused container with you to workspace"))
        // .arg(Arg::with_name(&"send_to")
        //     .short("-s")
        //     .long("--send-to")
        //     .help("Send focused container away to workspace"))
        // .group(ArgGroup::with_name("action")
        //     .arg("bring_to")
        //     .arg("send_to"))
        // .arg(Arg::with_name("toggle")
        //     .short("-t")
        //     .long("--toggle")
        //     .help("Skip menu & choose workspace 2 (default: false)"))
        // .arg(Arg::with_name("order")
        //     .short("-o")
        //     .long("--order")
        //     .possible_values(&["default", "last-used"])
        //     .default_value("default")
        //     .help("Method used to determine workspace display order"))
        .arg(Arg::with_name("action")
            .short("-a")
            .long("--action")
            .possible_values(&["go_to", "send_to", "bring_to", "toggle"])
            .default_value("go_to")
            .help("Action to perform"))
        // .arg(Arg::with_name("menu")
        //     .short("-m")
        //     .long("--menu")
        //     .possible_values(&["dmenu", "rofi"])
        //     .default_value("dmenu")
        //     .help("Program used to render the menu"))
        .get_matches();

    // ------------------------------------------------

    // define filename for fluidspaces IPC socket
    let socket_filename = "/tmp/fluidspaces.sock";

    // connect to the socket
    let mut stream = match UnixStream::connect(socket_filename) {
        Ok(sock) => sock,
        Err(e) => panic!("Couldn't connect to socket: {:?}", e),
    };

    // set stream behavior to "blocking"
    if let Err(e) = stream.set_nonblocking(false) {
        panic!("Couldn't set socket to blocking mode: {:?}", e);
    }

    // send the value of the action option to the socket
    stream
        .write_all(matches.value_of("action").unwrap_or("go_to").as_bytes())
        .unwrap();
}
