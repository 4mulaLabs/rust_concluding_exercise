use crossbeam::channel::{Receiver, Sender};

use crate::protocol::PublishedMessage;

#[derive(Clone)]
pub struct SubscriberEntry {
    pub topics: Vec<String>,
    pub sender: Sender<PublishedMessage>,
}

pub enum DispatcherEvent {
    Published(PublishedMessage),
    NewSubscriber(SubscriberEntry),
}

pub fn dispatcher_loop(receiver: Receiver<DispatcherEvent>) {
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

                let mut alive_subscribers = Vec::new();

                for subscriber in subscribers.into_iter() {
                    if subscriber.topics.contains(&topic_name) {
                        // try to send - if it fails update subscribers by removing subscriber
                        if subscriber.sender.send(message.clone()).is_ok() {
                            delivered_count += 1;
                            alive_subscribers.push(subscriber);
                        } else {
                            println!("removing dead subscriber for topic '{}'", topic_name);
                        }
                    } else {
                        // not interested → keep it
                        alive_subscribers.push(subscriber);
                    }
                }

                subscribers = alive_subscribers;

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