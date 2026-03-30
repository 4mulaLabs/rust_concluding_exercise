use anyhow::{Context, Result};
use crossbeam::channel::{unbounded, Receiver, Sender};
use std::net::{TcpListener, TcpStream};
use std::thread;

use crate::dispatcher::{DispatcherEvent, SubscriberEntry};
use crate::protocol::{
    read_publisher_packet, read_subscriber_topics, write_published_message, PublishedMessage,
};

pub fn run_publisher_listener(port: u16, dispatcher_sender: Sender<DispatcherEvent>) -> Result<()> {
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

pub fn run_subscriber_listener(
    port: u16,
    dispatcher_sender: Sender<DispatcherEvent>,
) -> Result<()> {
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

pub fn handle_publisher_client(mut stream: TcpStream, dispatcher_sender: Sender<DispatcherEvent>) {
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

pub fn handle_subscriber_client(
    mut stream: TcpStream,
    dispatcher_sender: Sender<DispatcherEvent>,
) {
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

fn subscriber_writer_thread(mut stream: TcpStream, receiver: Receiver<PublishedMessage>) {
    let peer = stream.peer_addr().ok();

    while let Ok(message) = receiver.recv() {
        if write_published_message(&mut stream, &message).is_err() {
            println!("subscriber disconnected while writing: {:?}", peer);
            break;
        }
    }
}