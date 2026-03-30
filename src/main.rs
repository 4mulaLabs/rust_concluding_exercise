use anyhow::Result;
use clap::Parser;
use crossbeam::channel::unbounded;
use std::thread;

mod client_handlers;
mod dispatcher;
mod protocol;

use client_handlers::{run_publisher_listener, run_subscriber_listener};
use dispatcher::{dispatcher_loop, DispatcherEvent};

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    pub_port: u16,

    #[arg(long)]
    sub_port: u16,
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