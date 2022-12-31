use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const LOCAL: &str = "127.0.0.1:6000";
// set buffer size of message
const MSG_SIZE: usize = 32;

fn sleep() {
    thread::sleep(Duration::from_millis(100));
}

fn main() {
    // init listener for incoming TCP connections
    let listener = TcpListener::bind(LOCAL).expect("Listener failed to bind");
    listener
        .set_nonblocking(true)
        .expect("Failed to set non-blocking");

    let mut clients = vec![];

    // pass to transmitter and receiver
    let (tx, rx) = mpsc::channel::<String>();
    loop {
        if let Ok((mut socket, addr)) = listener.accept() {
            println!("Client {} connected", addr);
            let tx = tx.clone();
            clients.push(socket.try_clone().expect("Failed to clone client"));

            thread::spawn(move || loop {
                let mut buff = vec![0; MSG_SIZE];

                match socket.read_exact(&mut buff) {
                    Ok(_) => {
                        // convert message into an iterator
                        // collect all non-whitespace character into vector
                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        // convert slice of strings into string
                        let msg = String::from_utf8(msg).expect("Invalig utf-8 message");

                        println!("{}: {:?}", addr, msg);
                        tx.send(msg).expect("Failed to send message to rx");
                    }
                    // close connection if we receive an error that would block
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("Closing connection with: {}", addr);
                        break;
                    }
                }
                // thread sleeps between each loop
                sleep();
            });
        }
        // when server receives a message
        if let Ok(msg) = rx.try_recv() {
            clients = clients
                .into_iter()
                .filter_map(|mut client| {
                    // convert messages into bytes
                    let mut buff = msg.clone().into_bytes();
                    buff.resize(MSG_SIZE, 0);

                    // write the buffer and collect to vector
                    client.write_all(&buff).map(|_| client).ok()
                })
                .collect::<Vec<_>>();
        }
        sleep();
    }
}
