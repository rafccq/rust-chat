use std::thread;
use std::error::Error;

use tokio;
use tokio::net::{TcpListener, TcpStream};

mod codec;

mod common;
use common::*;

use clap::{Arg, App};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (host, port, mode) = read_args();
    let addr = format!("{}:{}", host, port);

    match &mode[..] {
        "server" => run_server(&addr, &port).await?,
        "client" => run_client(&addr).await?,
        _ => { }
    }

    Ok(())
}

async fn run_server(addr: &str, port: &str) -> Result<(), Box<dyn Error>> {
    let mut listener = TcpListener::bind(addr).await?;

    println!("Server started on port {}", port);

    loop {
        println!("Listening...",);
        let (stream, addr) = listener.accept().await?;
        println!("New connection: {}", addr);

        match process_connection(stream).await {
            // client disconnected, just wait for next client
            ProcessResult::PeerDisconnected => println!("Client disconnected"),

            // user quit: just exit
            ProcessResult::Error | ProcessResult::UserQuit => {
                break
            }
        }
    }

    drop(listener);
    Ok(())
}

async fn run_client(addr: &str) -> Result<(), Box<dyn Error>> {
    let stream = TcpStream::connect(addr).await?;
    println!("Successfully connected to {}", addr);

    match process_connection(stream).await {
        ProcessResult::PeerDisconnected => {
            println!("Server disconnected");
            // somehow, stdin is still stuck at this point, waiting for input, so manually exit
            std::process::exit(0);
        },
        // Currently all errors are handled inside process_connection()
        _ => {}
    }
    Ok(())
}

pub fn read_args() -> (String, String, String) {
    let matches = App::new("ahref application: 1-1 chat")
        .version("0.1")
        .author("Rafael C. <rafa.gdev@gmail.com>")
        .about("")
        .arg(Arg::with_name("mode")
            .short("m")
            .long("mode")
            .value_name("MODE")
            .help("server/client")
            .takes_value(true)
            .possible_values(&["client", "server"]))
        .arg(Arg::with_name("host")
            .short("h")
            .long("host")
            .value_name("addr")
            .help("IP address to listen for connections (server) or connect to (client)")
            .takes_value(true))
        .arg(Arg::with_name("port")
            .short("p")
            .long("port")
            .value_name("number")
            .help("Port to listen for connections (server) or connect to (client)")
            .takes_value(true))
        .get_matches();

    let host = matches.value_of("host").unwrap_or("127.0.0.1");
    let port = matches.value_of("port").unwrap_or("6142");
    let mode = matches.value_of("mode").unwrap_or("server");

    (host.to_string(), port.to_string(), mode.to_string())
}
