extern crate clap;
extern crate term;
extern crate ws;

use std::io;
use std::io::prelude::*;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender as TSender;
use std::thread;

use tokio::time;
use ws::{connect, CloseCode, Error, ErrorKind, Handler, Handshake, Message, Result, Sender};

pub async fn ws_connect() {
    let url = "ws://127.0.0.1:3012".to_string();

    let (tx, rx) = channel();

    // Run client thread with channel to give it's WebSocket message sender back to us
    let client = thread::spawn(move || {
        println!("Connecting to {}", url);
        connect(url, |sender| Client {
            ws_out: sender,
            thread_out: tx.clone(),
        })
        .unwrap();
    });

    if let Ok(Event::Connect(sender)) = rx.recv() {
        // If we were able to connect, print the instructions
        println!("connect success");

        // Main loop
        loop {
            // Get user input
            // let mut input = String::new();
            // io::stdin().read_line(&mut input).unwrap();

            let mut interval = time::interval(time::Duration::from_secs(100));
            interval.tick().await;
            if let Ok(Event::Disconnect) = rx.try_recv() {
                break;
            }

            // Send the message
            display(&format!(">>> {}","666"));
            // sender.send(input.trim()).unwrap();
            sender.send("666").unwrap();
        }
    }
    // Ensure the client has a chance to finish up
    client.join().unwrap();
}

fn display(string: &str) {
    let mut view = term::stdout().unwrap();
    view.carriage_return().unwrap();
    view.delete_line().unwrap();
    println!("{}", string);
    print!("?> ");
    io::stdout().flush().unwrap();
}


struct Client {
    ws_out: Sender,
    thread_out: TSender<Event>,
}

impl Handler for Client {
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        self.thread_out
            .send(Event::Connect(self.ws_out.clone()))
            .map_err(|err| {
                Error::new(
                    ErrorKind::Internal,
                    format!("Unable to communicate between threads: {:?}.", err),
                )
            })
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        display(&format!("<<< {}", msg));
        Ok(())
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        if reason.is_empty() {
            display(&format!(
                "<<< Closing<({:?})>\nHit any key to end session.",
                code
            ));
        } else {
            display(&format!(
                "<<< Closing<({:?}) {}>\nHit any key to end session.",
                code, reason
            ));
        }

        if let Err(err) = self.thread_out.send(Event::Disconnect) {
            display(&format!("{:?}", err))
        }
    }

    fn on_error(&mut self, err: Error) {
        display(&format!("<<< Error<{:?}>", err))
    }
}

enum Event {
    Connect(Sender),
    Disconnect,
}
