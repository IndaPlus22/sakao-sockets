use std::{
    fs,
    io::{self, prelude::*, BufReader, ErrorKind},
    net::{TcpListener, TcpStream},
    sync::{mpsc::{self, TryRecvError}, Arc, Mutex},
    thread,
    time::Duration, string,
};

/* Address to server. */
const SERVER_ADR: &str = "127.0.0.1:7878";

/* Max message size in characters. */
const MSG_SIZE: usize = 32;

fn main() {
    let username = "bruh";

    chat(username.to_string());
}

fn chat(username: String) {
    let _username = username.to_owned();
    let mut client = connect(username);
    // prevent io stream operation from blocking sockets in case of slow communication
    client
        .set_nonblocking(true)
        .expect("Failed to initiate non-blocking!");

    // create channel for communication between threads
    let (sender, receiver) = mpsc::channel::<String>();

    let _username = _username.to_owned();
    /* Start thread that listens to server. */
    thread::spawn(move || loop {
        let mut msg_buffer = vec![0; MSG_SIZE];

        /* Read message from server. */
        match client.read_exact(&mut msg_buffer) {
            // received message
            Ok(_) => {
                // read until end-of-message (zero character)
                let _msg = msg_buffer
                    .into_iter()
                    .take_while(|&x| x != 0)
                    .collect::<Vec<_>>();
                let msg = String::from_utf8(_msg).expect("Invalid UTF-8 message!");

                println!("{}: Message: {:?}", _username, msg);
            }
            // no message in stream
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            // connection error
            Err(_) => {
                println!("Lost connection with server!");
                break;
            }
        }

        /* Send message in channel to server. */
        match receiver.try_recv() {
            // received message from channel
            Ok(msg) => {
                let mut msg_buffer = msg.clone().into_bytes();
                // add zero character to mark end of message
                msg_buffer.resize(MSG_SIZE, 0);

                if client.write_all(&msg_buffer).is_err() {
                    println!("Failed to send message!")
                }
            }
            // no message in channel
            Err(TryRecvError::Empty) => (),
            // channel has been disconnected (main thread has terminated)
            Err(TryRecvError::Disconnected) => break,
        }

        thread::sleep(Duration::from_millis(100));
    });

    /* Listen for and act on user messages. */
    println!("Chat open:");
    loop {
        let mut msg_buffer = String::new();

        // wait for user to write message
        io::stdin().read_line(&mut msg_buffer).expect("Failed to read user message!");

        let msg = msg_buffer.trim().to_string();

        // quit on message ":quit" or on connection error
        if msg == ":quit" || sender.send(msg).is_err() {break}
    }

    println!("Closing chat...");
}

fn connect(username: String) -> TcpStream{
    // connect to server
    let mut client = match TcpStream::connect(SERVER_ADR) {
        Ok(_client) => {
            println!("Connected to server at: {}", SERVER_ADR);
            _client
        }
        Err(_) => {
            println!("Failed to connect to server at: {}", SERVER_ADR);
            std::process::exit(1)
        }
    };

    client
}