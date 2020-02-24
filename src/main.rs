use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::io::Write;

use tokio;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::signal;

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

async fn read_console_input(mut tx: mpsc::Sender<InternalMessage>, client_connected: &AtomicBool) -> Result<(), Box<dyn Error>> {
    let mut input = String::new();

    loop {
        match std::io::stdin().read_line(&mut input) {
            Err(error) => println!("Unable to read from the console: {}", error),
            Ok(_n) => {
                input.pop();
                if client_connected.load(Ordering::Relaxed) {
                    // process commands: currently only \quit is implemented
                    if input == "\\quit" {
                        tx.try_send(InternalMessage::Quit);
                        break;
                    }

                    tx.try_send(InternalMessage::Input(input.clone()));
               }
            }
        }
        input.clear();
    }

    Ok(())
}

async fn run_server(addr: &str, port: &str) -> Result<(), Box<dyn Error>> {
    let mut listener = TcpListener::bind(addr).await?;

    let shared = Arc::new(AtomicBool::new(false));
    let client_connected = Arc::clone(&shared);

    println!("Server started on port {}", port);
    
    let (tx, mut rx) = mpsc::channel(1);
    let tx_input = tx.clone();
    let mut tx_signal = tx.clone();

    tokio::spawn(async move { read_console_input(tx_input, &client_connected).await; });

    loop {
        println!("Listening...");

        let (stream, addr) = listener.accept().await?;
        println!("New connection: {}", addr);
        
        let client_connected = Arc::clone(&shared);
        
        client_connected.store(true, Ordering::Relaxed);

        match process_connection(stream, tx.clone(), &mut rx).await {
            // client disconnected, just wait for next client
            ProcessResult::PeerDisconnected => {
                println!("Client disconnected");
                client_connected.store(false, Ordering::Relaxed);
            },

            // user quit: just exit
            ProcessResult::Error | ProcessResult::UserQuit => {
                client_connected.store(false, Ordering::Relaxed);
                break;
            },
            ProcessResult::Terminated => {
                client_connected.store(false, Ordering::Relaxed);
                break;
            },
        }
    }

    drop(listener);
    Ok(())
}

async fn run_client(addr: &str) -> Result<(), Box<dyn Error>> {
    let stream = TcpStream::connect(addr).await?;
    println!("Successfully connected to {}", addr);
    
    let (tx, mut rx) = mpsc::channel(1);
    let mut tx_input = tx.clone();
    let mut tx_signal = tx.clone();

    let shared = Arc::new(AtomicBool::new(true));
    let server_connected = Arc::clone(&shared);

    // SIGINT signal handler
    tokio::spawn(async move {
        signal::ctrl_c().await;
        tx_signal.try_send(InternalMessage::Terminated);
    });

    // client console input thread:
    tokio::spawn(async move {
        let mut input = String::new();

        loop {
            match std::io::stdin().read_line(&mut input) {
                Err(error) => println!("Unable to read from the console: {}", error),
                Ok(_n) => {
                    input.pop();

                    if !server_connected.load(Ordering::Relaxed) {
                        break;
                    }

                    if input == "\\quit" {
                        tx_input.try_send(InternalMessage::Quit);
                        break;
                    }
                    
                    tx_input.try_send(InternalMessage::Input(input.clone()));
                }
            }
            input.clear();
        }
    });

    match process_connection(stream, tx.clone(), &mut rx).await {
        ProcessResult::PeerDisconnected => {
            print!("Disconnected from server, press any key to exit... ");
            std::io::stdout().flush();

            // signal the console input thread that the server was disconnected
            let client_connected = Arc::clone(&shared);
            client_connected.store(false, Ordering::Relaxed);
        },
        ProcessResult::Terminated => {
            print!("\nPress any key to exit... ");
            std::io::stdout().flush();

            // signal the console input thread that the server was disconnected
            let client_connected = Arc::clone(&shared);
            client_connected.store(false, Ordering::Relaxed);
        },
        // Currently all errors are handled inside process_connection()
        _ => {}
    }
    Ok(())
}

pub fn read_args() -> (String, String, String) {
    let matches = App::new("A simple 1-1 chat application")
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
