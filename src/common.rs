use std::time::{Instant};

use futures::stream::{self, StreamExt};
use futures_util::sink::SinkExt;

use bytes::{BytesMut};

use tokio;
use tokio::sync::mpsc;
use tokio::net::{TcpStream};
use tokio_util::codec::{Framed};

use super::codec;
use codec::ChatMessage;
use codec::BytesCodecExt;

pub enum InternalMessage {
    Input(String),
    Message(String),
    ACK,
    Disconnected,
    Terminated,
    Quit,
    Error
}

pub enum ProcessResult {
    PeerDisconnected,
    UserQuit,
    Terminated,
    Error
}

pub async fn process_connection(stream: TcpStream, tx_messages: mpsc::Sender<InternalMessage>, rx: &mut mpsc::Receiver<InternalMessage>) -> ProcessResult {
    let (mut sender, receiver) = Framed::new(stream, BytesCodecExt::new()).split();

    // thread to receive incoming messages from the network
    tokio::spawn(async move { recv_incoming_messages(receiver, tx_messages).await; });

    let (mut sender_tx, _sender_rx) = mpsc::channel(1);

    // receiver loop
    loop {
        let mut start = Instant::now();

        match rx.recv().await {
            Some(InternalMessage::Message(msg)) => {
                println!("[Message] {}", msg);
                sender.send(ChatMessage::ACK).await;
            },
            Some(InternalMessage::Input(msg)) => {
                start = Instant::now();

                let mut buf = BytesMut ::new();
                buf.extend_from_slice(msg.as_bytes());
                sender.send(ChatMessage::Message(buf)).await;
            }
            Some(InternalMessage::ACK) => {
                let duration = start.elapsed();
                println!("Ack received after {:?}", duration);
                sender_tx.try_send(InternalMessage::ACK);
            },
            Some(InternalMessage::Quit) => {
                sender.send(ChatMessage::Disconnected).await;
                return ProcessResult::UserQuit;
            },
            Some(InternalMessage::Terminated) => {
                sender.send(ChatMessage::Disconnected).await;
                return ProcessResult::Terminated;
            },
            Some(InternalMessage::Disconnected) => {
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
