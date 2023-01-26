mod threadpool;

use std::{
    fs,
    io::{prelude::*, BufReader, ErrorKind},
    net::{SocketAddr, TcpListener, TcpStream},
    os::windows::prelude::AsSocket,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};

use threadpool::ThreadPool;

const SERVER_ADR: &str = "127.0.0.1:7878";

fn main() {
    // let listener = TcpListener::bind(SERVER_ADR).unwrap();
    // connect to server
    let server = match TcpListener::bind(SERVER_ADR) {
        Ok(_client) => {
            println!("Opened server at: {}", SERVER_ADR);
            _client
        }
        Err(_) => {
            println!("Failed to connect to socket at: {}", SERVER_ADR);
            std::process::exit(1)
        }
    };

    // prevent io stream operation from blocking sockets in case of slow communication
    server
        .set_nonblocking(true)
        .expect("Failed to initiate non-blocking!");

    let mut clients = vec![];

    let pool = ThreadPool::new(4);

    // create channel for communication between threads
    let (sender, receiver) = mpsc::channel::<String>();

    loop {
        if let Ok((mut socket, addr)) = server.accept() {
            println!("someone here {}", addr);

            clients.push(socket.try_clone().unwrap());

            pool.execute(|| {
                handle_connection(socket);
                println!("okokok");
            });
        }
    }

    // println!("Shutting down!");
}

fn handle_connection(mut socket: TcpStream) {
    let mut test = vec![0; 32];

    match socket.read_exact(&mut test) {
        Ok(_) => {
            // read until end-of-message (zero character)
            let _msg = test
                .into_iter()
                .take_while(|&x| x != 0)
                .collect::<Vec<_>>();
            let msg = String::from_utf8(_msg).expect("Invalid UTF-8 message!");

            println!("msg: {:?}", msg);
        }
        Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
        Err(_) => {
            println!("closing");
        }
    }
}

fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(100));
}
