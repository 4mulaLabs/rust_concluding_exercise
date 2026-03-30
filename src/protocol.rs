use anyhow::{Context, Result};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};
use std::net::TcpStream;

#[derive(Debug, Clone)]
pub struct PublishedMessage {
    pub topic: String,
    pub data: Vec<u8>,
}

pub fn read_publisher_packet(stream: &mut TcpStream) -> Result<PublishedMessage> {
    let topic_len = stream.read_u32::<BigEndian>()? as usize;

    let mut topic_bytes = vec![0u8; topic_len];
    stream.read_exact(&mut topic_bytes)?;

    let topic =
        String::from_utf8(topic_bytes).context("publisher sent topic that is not valid UTF-8")?;

    let msg_len = stream.read_u32::<BigEndian>()? as usize;

    let mut data = vec![0u8; msg_len];
    stream.read_exact(&mut data)?;

    Ok(PublishedMessage { topic, data })
}

pub fn read_subscriber_topics(stream: &mut TcpStream) -> Result<Vec<String>> {
    let topic_count = stream.read_u32::<BigEndian>()? as usize;

    let mut topics = Vec::new();

    for _ in 0..topic_count {
        let topic_len = stream.read_u32::<BigEndian>()? as usize;

        let mut topic_bytes = vec![0u8; topic_len];
        stream.read_exact(&mut topic_bytes)?;

        let topic = String::from_utf8(topic_bytes)
            .context("subscriber sent topic that is not valid UTF-8")?;

        topics.push(topic);
    }

    Ok(topics)
}

pub fn write_published_message(stream: &mut TcpStream, message: &PublishedMessage) -> Result<()> {
    let topic_bytes = message.topic.as_bytes();

    stream.write_u32::<BigEndian>(topic_bytes.len() as u32)?;
    stream.write_all(topic_bytes)?;

    stream.write_u32::<BigEndian>(message.data.len() as u32)?;
    stream.write_all(&message.data)?;

    Ok(())
}