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

/* Max message size in characters. */
const MSG_SIZE: usize = 32;

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
    // server
    //     .set_nonblocking(true)
    //     .expect("Failed to initiate non-blocking!");

    let mut clients: Vec<TcpStream> = vec![];

    let pool: ThreadPool = ThreadPool::new(4);

    loop {
        let client: (TcpStream, SocketAddr) = server.accept().unwrap();

        let addr = client.1;
        println!("someone here {}", addr);

        clients.push(client.0.try_clone().unwrap());

        pool.execute(|| {
            handle_connection(client.0);
            println!("okokok");
        });
    }

    // println!("Shutting down!");
}

fn handle_connection(mut socket: TcpStream) {
    loop {
        let mut msg_buff = vec![0; MSG_SIZE];

        match socket.read_exact(&mut msg_buff) {
            Ok(_) => {
                // read until end-of-message (zero character)
                let _msg: Vec<u8> = msg_buff
                    .into_iter()
                    .take_while(|&x| x != 0)
                    .collect::<Vec<_>>();
                let msg: String = String::from_utf8(_msg).expect("Invalid UTF-8 message!");

                println!("msg: {:?}", msg);

                // sends back the message as response to check if the client is still connected
                // let response = format!("sent: {}", msg);
                // stream.write_all(response.as_bytes()).unwrap();
            }
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("closing connection with: {}", socket.peer_addr().unwrap());
                break;
            }
        }

        sleep();
    }
}

fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(100));
}
