use std::io;
use std::error::Error;
use std::time::{Instant};

use futures::future::FutureExt;
use futures::stream::{self, StreamExt};
use futures_util::sink::SinkExt;

use bytes::{BytesMut};

use tokio;
use tokio::sync::mpsc;
use tokio::net::{TcpStream};
use tokio_util::codec::{Framed};
use tokio::signal;

use super::codec;
use codec::ChatMessage;
use codec::BytesCodecExt;

enum InternalMessage {
    Input(String),
    Message(String),
    ACK,
    Disconnected,
    Quit,
    Error
}

pub enum ProcessResult {
    PeerDisconnected,
    UserQuit,
    Error
}

pub async fn process_connection(stream: TcpStream) -> ProcessResult {
    let (tx_input, mut rx) = mpsc::channel(1);
    let tx_messages = tx_input.clone();


    let (mut sender, receiver) = Framed::new(stream, BytesCodecExt::new()).split();

    // thread to receive incoming messages from the network
    tokio::spawn(async move { recv_incoming_messages(receiver, tx_messages).await; });

    let (sender_tx, mut sender_rx) = mpsc::channel(1);
    let mut sender_tx = sender_tx.clone();

    // sender thread: keyboard input
    let (future, handle_input) = async move {
        read_console_input(tx_input, &mut sender_rx).await;
    }.remote_handle();

    tokio::spawn(future);

    // receiver loop
    loop {
        match rx.recv().await {
            Some(InternalMessage::Message(msg)) => {
                println!("[Message] {}", msg);
                sender.send(ChatMessage::ACK).await;
            },
            Some(InternalMessage::Input(msg)) => {
                let mut buf = BytesMut ::new();
                buf.extend_from_slice(msg.as_bytes());
                sender.send(ChatMessage::Message(buf)).await;
            }
            Some(InternalMessage::ACK) => {
                sender_tx.try_send(InternalMessage::ACK);
            },
            Some(InternalMessage::Quit) => {
                sender.send(ChatMessage::Disconnected).await;
                // drop the input thread before exiting
                drop(handle_input);
                return ProcessResult::UserQuit;
            },
            Some(InternalMessage::Disconnected) => {
                // drop the input thread before exiting
                drop(handle_input);
                return ProcessResult::PeerDisconnected;
            },
            Some(InternalMessage::Error) | None => {
                println!("Something wrong happened");
                return ProcessResult::Error;
            },
        }
    }
}

async fn recv_incoming_messages(mut receiver: stream::SplitStream<Framed<TcpStream, BytesCodecExt>>, mut tx: mpsc::Sender<InternalMessage>) {
    while let Some(m) = receiver.next().await {
        match m {
            Ok(ChatMessage::Message(mut buf)) => {
                let bytes = buf.split_to(buf.len());
                let msg = std::str::from_utf8(&bytes).unwrap();
                tx.try_send(InternalMessage::Message(msg.to_string()));
            },
            Ok(ChatMessage::ACK) => {
                tx.try_send(InternalMessage::ACK);
            },
            Ok(ChatMessage::Disconnected) => {
                tx.try_send(InternalMessage::Disconnected);
                break
            },
            Err(_) => {
                println!("Unable to receive message from the server");
                tx.try_send(InternalMessage::Error);
                break
            }
        }
    }
}

async fn read_console_input(mut tx: mpsc::Sender<InternalMessage>, rx_ack: &mut mpsc::Receiver<InternalMessage>) -> Result<(), Box<dyn Error>> {
    let mut input = String::new();

    loop {
        match io::stdin().read_line(&mut input) {
            Err(error) => println!("error: {}", error),
            Ok(_n) => {
                input.pop();

                // process commands: currently only \quit is available
                if input == "\\quit" {
                    tx.try_send(InternalMessage::Quit);
                    break;
                }

                let start = Instant::now();
                tx.try_send(InternalMessage::Input(input.clone()));

                match rx_ack.recv().await {
                    Some(InternalMessage::ACK) => {
                        let duration = start.elapsed();
                        println!("Ack received after {:?}", duration);
                    },
                    Some(_) => { },
                    None | _ => {
                        println!("Unable to read from the console.");
                        tx.try_send(InternalMessage::Error);
                        break;
                    }
                }
            }
        }
        input.clear();
    }

    Ok(())
}
