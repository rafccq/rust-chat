use bytes::{BufMut, BytesMut};
use tokio_util::codec::{Encoder, Decoder};

use std::io;

pub enum ChatMessage {
    Message(BytesMut),
    Disconnected,
    ACK,
}

fn byte_value(m: &ChatMessage) -> u8 {
    match m {
        ChatMessage::ACK => 0x0A,
        ChatMessage::Message(_) => 0x0B,
        ChatMessage::Disconnected => 0x0C,
    }
}

// A simple extension of "BytesCodec" from tokio. Each message contains a header indicating the type,
// followed by a byte buffer with the message contents
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct BytesCodecExt(());

impl BytesCodecExt {
    pub fn new() -> BytesCodecExt {
        BytesCodecExt(())
    }
}

impl Decoder for BytesCodecExt {
    type Item = ChatMessage;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<ChatMessage>, io::Error> {
        if buf.is_empty() {
            return Ok(None);
        }

        // the header is the first byte
        let header = buf[0];

        let len = buf.len();
        let mut contents = buf.split_to(len);   // make sure we consume the buffer

        // contents start after the first byte
        let contents = contents.split_off(1);

        if header == byte_value(&ChatMessage::ACK) {
            return Ok(Some(ChatMessage::ACK));
        }

        if header == byte_value(&ChatMessage::Disconnected) {
            return Ok(Some(ChatMessage::Disconnected));
        }

        // if we reached here, its a "Message" type
        Ok(Some(ChatMessage::Message(contents)))
    }
}

impl Encoder for BytesCodecExt {
    type Item = ChatMessage;
    type Error = io::Error;

    fn encode(&mut self, data: ChatMessage, buf: &mut BytesMut) -> Result<(), io::Error> {
        let header = byte_value(&data);
        match data {
            // this message is just one byte
            ChatMessage::ACK | ChatMessage::Disconnected => {
                buf.reserve(1);
                buf.put_u8(header);
            },
            // write the header, followed by the contents of the message
            ChatMessage::Message(msg) => {
                buf.reserve(msg.len() + 1);
                buf.put_u8(header);
                buf.put(msg);
            },
        }

        Ok(())
    }
}
