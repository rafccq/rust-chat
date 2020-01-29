//use std::net::{TcpStream};
use std::io::{self, Read, Write};
use std::str::from_utf8;


use tokio::prelude::*;
//use tokio_util::codec::{Framed, LengthDelimitedCodec, BytesCodec};
use tokio_util::codec::{Framed, BytesCodec, FramedRead, FramedWrite,LengthDelimitedCodec};
use futures::SinkExt;
use bytes::{BytesMut};
use tokio::net::{TcpStream};
use std::error::Error;
use std::net::Shutdown;

mod codec;
use self::codec::{BytesCodecExt, ChatMessage};

//#[path = "common.rs"]
//mod common;
//use common::ChatMessage;
//mod common;
//use common;

/*
fn main() {
    let addr = "127.0.0.1:6142";
    let mut tcp = TcpStream::connect(addr).await;
//    let (r, w) = stream.split();

    println!("Successfully connected to {}", addr);

    // Once we connect, send a `Handshake` with our name.
    let handshake = tcp.and_then(|stream| {
        let handshake_io = stream.framed(BytesCodecExt::new());

        // After sending the handshake, convert the framed stream back into its inner socket.
        handshake_io.send(Handshake::new(name)).map(|handshake_io| handshake_io.into_inner())
    });
}
*/

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = "127.0.0.1:6144";
    let mut stream = TcpStream::connect(addr).await?;
    let (r, w) = stream.split();

    println!("Successfully connected to {}", addr);

    let mut transport = FramedWrite::new(w, BytesCodecExt::new());
//    let (r, w) = transport.split();

    let mut input = String::new();

    loop {
        match io::stdin().read_line(&mut input) {
            Err(error) => println!("error: {}", error),
            Ok(n) => {
                input.pop();
                println!("{} bytes read", n);
//                stream.write(input.as_bytes()).unwrap();
//                input.pop();

                // ---------------
//                let frame = Bytes::from("hello world");
//                let frame = Bytes::from(input.clone().into_bytes());
//                transport.send(frame).await?;
                /*
                let mut buf:BytesMut = BytesMut::with_capacity(input.len());
                let i:u8 = buf;
                let bytes = input.bytes();

                for byte in bytes {
//                    let s:String = byte;
                    buf.put_u8(b'h');
                }
                */
                let inputClone = input.clone();
                let mut buf = bytes::BytesMut::new();
                let bytes = &inputClone.into_bytes()[..];
//                let i:u16 = bytes;
                buf.extend_from_slice(bytes);

                transport.send(ChatMessage::MESSAGE(buf)).await;
                // ---------------
                print!("{}", input);
            }
        }
        input.clear();
//                println!(">loop<");
    }

//    transport.shutdown();

    Ok(())
}

/*
fn main2() {
    match TcpStream::connect("127.0.0.1:6142") {
        Ok(mut stream) => {
            println!("Successfully connected to server in port 6142");

            let msg = b"Hello!";

            let mut input = String::new();

            loop {
                match io::stdin().read_line(&mut input) {
                    Err(error) => println!("error: {}", error),
                    Ok(n) => {
                        input.pop();
//                        println!("{} bytes read", n);
//                        stream.write(input.as_bytes()).unwrap();
//                        input.pop();

                        // ---------------
                        let mut transport = Framed::new(stream, LengthDelimitedCodec::new());
                        let frame = Bytes::from("hello world");

                        transport.send(frame);

                        // ---------------
                        print!("{}", input);
                    }
                }
                input.clear();
//                println!(">loop<");
            }

//            stream.write(msg).unwrap();
            println!("Sent Hello, awaiting reply...");


            let mut data = [0 as u8; 6]; // using 6 byte buffer
            match stream.read_exact(&mut data) {
                Ok(_) => {
                    if &data == msg {
                        println!("Reply is ok!");
                    } else {
                        let text = from_utf8(&data).unwrap();
                        println!("Unexpected reply: {}", text);
                    }
                },
                Err(e) => {
                    println!("Failed to receive data: {}", e);
                }
            }
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Terminated.");
}
*/
