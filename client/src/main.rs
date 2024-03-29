use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

// based on: https://www.youtube.com/watch?v=CIhlfJSvxe4&list=WL&index=16&t=1s&ab_channel=TensorProgramming

const LOCAL: &str = "127.0.0.1:6000";
// set buffer size of message
const MSG_SIZE: usize = 32;

fn main() {
    let mut client = TcpStream::connect(LOCAL).expect("Stream failed to connect");
    client.set_nonblocking(true).expect("Failed to set non-blocking");

    // pass to transmitter and receiver
    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || loop {
        // buffer defined as a vector of 0s of size message size
        let mut buff = vec![0; MSG_SIZE];
        match client.read_exact(&mut buff) {
            Ok(_) => {
                // collect non-zero data
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                println!("Message received: {:?}", msg);
            },
            // handle WouldBlock error
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            // handle other type of error
            Err(_) => {
                println!("Server connection faled!");
                break;
            }
        }
        match rx.try_recv() {
            Ok(msg) => {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                // write buffers to client
                client.write_all(&buff).expect("Writing to socket failed");
                println!("Message sent: {:?}", msg);
            },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break
        }
        thread::sleep(Duration::from_millis(100));
    });

    println!("Write a Message: ");
    loop {
        let mut buff = String::new();
        // receive user input
        io::stdin().read_line(&mut buff).expect("Reading from user input failed");
        let msg = buff.trim().to_string();
        // handle exit conditions of the loop
        if msg == ":quit" || tx.send(msg).is_err() {
            break
        }
        println!("Application exiting...")
    }
}
