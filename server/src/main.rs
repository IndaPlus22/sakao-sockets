mod threadpool;

use std::{
    fs,
    io::{prelude::*, BufReader, ErrorKind},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
    sync::{mpsc, Arc, Mutex}
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
        },
        Err(_) => {
            println!("Failed to connect to socket at: {}", SERVER_ADR);
            std::process::exit(1)
        }
    };

    // prevent io stream operation from blocking sockets in case of slow communication
    server.set_nonblocking(true).expect("Failed to initiate non-blocking!");

    let mut clients = vec![];

    // create channel for communication between threads
    let (sender, receiver) = mpsc::channel::<String>();

    loop {
        /* Start listening thread on new connecting client. */
        if let Ok((mut socket, addr)) = server.accept() {

            println!("Client {} connected.", addr);

            let _sender = sender.clone();

            clients.push(
                socket.try_clone().expect("Failed to clone client! Client wont receive messages!"));

            thread::spawn(move || loop {

                let mut msg_buff = vec![0; MSG_SIZE];

                /* Read and relay message from client. */
                match socket.read_exact(&mut msg_buff) {
                    // received message
                    Ok(_) => {
                        let _msg = msg_buff
                            .into_iter()
                            .take_while(|&x| x != 0)
                            .collect::<Vec<_>>();
                        let msg = String::from_utf8(_msg).expect("Invalid UTF-8 message!");

                        println!("{}: {:?}", addr, msg);

                        _sender.send(msg).expect("Failed to relay message!");
                    }, 
                    // no message in stream
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    // connection error
                    Err(_) => {
                        println!("Closing connection with: {}", addr);
                        break;
                    }
                }

                sleep();
            });
        }

        /* Broadcast incoming messages. */
        if let Ok(msg) = receiver.try_recv() {

            // send message to all clients
            clients = clients.into_iter().filter_map(|mut client| {
                let mut msg_buff = msg.clone().into_bytes();
                // add zero character to mark end of message
                msg_buff.resize(MSG_SIZE, 0);

                client.write_all(&msg_buff).map(|_| client).ok()
            }).collect::<Vec<_>>();
        }

        sleep();
    }

    // let pool = ThreadPool::new(4);

    // for stream in server.incoming() {
    //     let stream = stream.unwrap();

    //     pool.execute(|| {
    //         handle_connection(stream);
    //     });
    // }

    // println!("Shutting down!");
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    
    // let request_line = buf_reader.lines().next().unwrap().unwrap();

    // let (status_line, filename) = match &request_line[..] {
    //     "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "src/hello.html"),
    //     "GET /sleep HTTP/1.1" => {
    //         thread::sleep(Duration::from_secs(2));
    //         ("HTTP/1.1 200 OK", "src/hello.html")
    //     }
    //     _ => ("HTTP/1.1 404 NOT FOUND", "src/404.html")
    // };

    // let contents = fs::read_to_string(filename).unwrap();
    // let length = contents.len();

    // let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    // stream.write_all(response.as_bytes()).unwrap();
}

fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(100));
}