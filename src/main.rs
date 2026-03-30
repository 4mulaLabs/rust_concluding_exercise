use anyhow::{Context, Result};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use clap::Parser;
use crossbeam::channel::{unbounded, Receiver, Sender};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    pub_port: u16,

    #[arg(long)]
    sub_port: u16,
}

#[derive(Debug, Clone)]
struct PublishedMessage {
    topic: String,
    data: Vec<u8>,
}

#[derive(Clone)]
struct SubscriberEntry {
    topics: Vec<String>,
    sender: Sender<PublishedMessage>,
}

enum DispatcherEvent {
    Published(PublishedMessage),
    NewSubscriber(SubscriberEntry),
}

fn read_publisher_packet(stream: &mut TcpStream) -> Result<PublishedMessage> {
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

fn read_subscriber_topics(stream: &mut TcpStream) -> Result<Vec<String>> {
    let topic_count = stream.read_u32::<BigEndian>()? as usize;

    let mut topics = Vec::new();

    for _ in 0..topic_count {
        let topic_len = stream.read_u32::<BigEndian>()? as usize;

        let mut topic_bytes = vec![0u8; topic_len];
        stream.read_exact(&mut topic_bytes)?;

        let topic =
            String::from_utf8(topic_bytes).context("subscriber sent topic that is not valid UTF-8")?;

        topics.push(topic);
    }

    Ok(topics)
}

fn write_published_message(stream: &mut TcpStream, message: &PublishedMessage) -> Result<()> {
    let topic_bytes = message.topic.as_bytes();

    stream.write_u32::<BigEndian>(topic_bytes.len() as u32)?;
    stream.write_all(topic_bytes)?;

    stream.write_u32::<BigEndian>(message.data.len() as u32)?;
    stream.write_all(&message.data)?;

    Ok(())
}

fn handle_publisher_client(mut stream: TcpStream, dispatcher_sender: Sender<DispatcherEvent>) {
    let peer = stream.peer_addr().ok();
    println!("publisher connected: {:?}", peer);

    loop {
        match read_publisher_packet(&mut stream) {
            Ok(message) => {
                println!(
                    "publisher sent: topic='{}', {} bytes",
                    message.topic,
                    message.data.len()
                );

                if dispatcher_sender
                    .send(DispatcherEvent::Published(message))
                    .is_err()
                {
                    eprintln!("dispatcher is gone");
                    break;
                }
            }
            Err(_) => {
                println!("publisher disconnected: {:?}", peer);
                break;
            }
        }
    }
}

fn subscriber_writer_thread(mut stream: TcpStream, receiver: Receiver<PublishedMessage>) {
    let peer = stream.peer_addr().ok();

    while let Ok(message) = receiver.recv() {
        if write_published_message(&mut stream, &message).is_err() {
            println!("subscriber disconnected while writing: {:?}", peer);
            break;
        }
    }
}

fn handle_subscriber_client(mut stream: TcpStream, dispatcher_sender: Sender<DispatcherEvent>) {
    let peer = stream.peer_addr().ok();
    println!("subscriber connected: {:?}", peer);

    match read_subscriber_topics(&mut stream) {
        Ok(topics) => {
            println!("subscriber topics from {:?}: {:?}", peer, topics);

            let (subscriber_sender, subscriber_receiver) = unbounded::<PublishedMessage>();

            match stream.try_clone() {
                Ok(writer_stream) => {
                    thread::spawn(move || {
                        subscriber_writer_thread(writer_stream, subscriber_receiver);
                    });

                    let entry = SubscriberEntry {
                        topics,
                        sender: subscriber_sender,
                    };

                    if dispatcher_sender
                        .send(DispatcherEvent::NewSubscriber(entry))
                        .is_err()
                    {
                        eprintln!("dispatcher is gone");
                    }
                }
                Err(err) => {
                    eprintln!("failed to clone subscriber stream {:?}: {}", peer, err);
                }
            }
        }
        Err(err) => {
            eprintln!("failed to read subscriber topics from {:?}: {}", peer, err);
        }
    }
}

fn run_publisher_listener(port: u16, dispatcher_sender: Sender<DispatcherEvent>) -> Result<()> {
    let listener = TcpListener::bind(("0.0.0.0", port))
        .with_context(|| format!("failed to bind publisher listener on port {}", port))?;

    println!("publisher listener started on port {}", port);

    for stream_result in listener.incoming() {
        match stream_result {
            Ok(stream) => {
                let dispatcher_sender = dispatcher_sender.clone();
                thread::spawn(move || {
                    handle_publisher_client(stream, dispatcher_sender);
                });
            }
            Err(err) => {
                eprintln!("failed to accept publisher connection: {}", err);
            }
        }
    }

    Ok(())
}

fn run_subscriber_listener(port: u16, dispatcher_sender: Sender<DispatcherEvent>) -> Result<()> {
    let listener = TcpListener::bind(("0.0.0.0", port))
        .with_context(|| format!("failed to bind subscriber listener on port {}", port))?;

    println!("subscriber listener started on port {}", port);

    for stream_result in listener.incoming() {
        match stream_result {
            Ok(stream) => {
                let dispatcher_sender = dispatcher_sender.clone();
                thread::spawn(move || {
                    handle_subscriber_client(stream, dispatcher_sender);
                });
            }
            Err(err) => {
                eprintln!("failed to accept subscriber connection: {}", err);
            }
        }
    }

    Ok(())
}

fn dispatcher_loop(receiver: Receiver<DispatcherEvent>) {
    let mut subscribers: Vec<SubscriberEntry> = Vec::new();

    while let Ok(event) = receiver.recv() {
        match event {
            DispatcherEvent::Published(message) => {
                println!(
                    "dispatcher received published message: topic='{}', {} bytes",
                    message.topic,
                    message.data.len()
                );

                let topic_name = message.topic.clone();
                let mut delivered_count = 0;

                subscribers.retain(|subscriber| {
                    if !subscriber.topics.contains(&topic_name) {
                        return true;
                    }

                    match subscriber.sender.send(message.clone()) {
                        Ok(()) => {
                            delivered_count += 1;
                            true
                        }
                        Err(_) => {
                            println!("removing dead subscriber for topic '{}'", topic_name);
                            false
                        }
                    }
                });

                println!(
                    "dispatched message on topic '{}' to {} subscribers",
                    topic_name,
                    delivered_count
                );
            }
            DispatcherEvent::NewSubscriber(subscriber) => {
                println!("dispatcher stored subscriber: {:?}", subscriber.topics);
                subscribers.push(subscriber);
                println!("subscriber count is now {}", subscribers.len());
            }
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!(
        "starting pubsub server: pub_port={}, sub_port={}",
        args.pub_port, args.sub_port
    );

    let (dispatcher_sender, dispatcher_receiver) = unbounded::<DispatcherEvent>();

    let dispatcher_thread = thread::spawn(move || {
        dispatcher_loop(dispatcher_receiver);
    });

    let pub_port = args.pub_port;
    let sub_port = args.sub_port;

    let pub_sender = dispatcher_sender.clone();
    let sub_sender = dispatcher_sender.clone();

    let pub_thread = thread::spawn(move || run_publisher_listener(pub_port, pub_sender));
    let sub_thread = thread::spawn(move || run_subscriber_listener(sub_port, sub_sender));

    pub_thread
        .join()
        .expect("publisher listener thread panicked")?;
    sub_thread
        .join()
        .expect("subscriber listener thread panicked")?;

    drop(dispatcher_sender);

    dispatcher_thread
        .join()
        .expect("dispatcher thread panicked");

    Ok(())
}