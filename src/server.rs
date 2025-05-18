use std::collections::HashMap;
use std::fmt;
use std::fs::{self};
use std::io::{prelude::*, BufReader};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};
use Server::ThreadPool;
struct Sensitive<T> {
    inner: T,
}

const ChildMode: bool = true;
impl<T: fmt::Display> fmt::Display for Sensitive<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if ChildMode == true {
            writeln!(f, "[Redacted]")
        } else {
            writeln!(f, "{inner}", inner = self.inner)
        }
    }
}

enum message {
    Connected {
        ip: SocketAddr,
        stream: Arc<TcpStream>,
    },
    unConnected {
        ip: SocketAddr,
        stream: Arc<TcpStream>,
    },
    New_message {
        ip: SocketAddr,
        str: String,
    },
}
#[derive(Clone)]
struct Banned {
    time: SystemTime,
    isbanned: bool,
}
//remember the clients' info
#[derive(Clone)]
struct Client {
    stream: Arc<TcpStream>,
    last_msg_time: SystemTime,
    strike_count: u32,
}
fn client(stream: Arc<TcpStream>, messages: Sender<message>) -> () {
    let ip: SocketAddr = stream.as_ref().peer_addr().unwrap();
    let _ = messages.send(message::Connected {
        ip,
        stream: stream.clone(),
    });
    loop {
        let mut buffer = vec![0; 100];
        //read from the stream
        //why as_ref? maybe bc now stream is a pointer
        let n = stream.as_ref().read(&mut buffer).unwrap();
        //let ip = stream.peer_addr().unwrap();
        let message_content = std::str::from_utf8(&buffer).unwrap();
        if n > 0 {
            messages
                .send(message::New_message {
                    ip,
                    str: message_content.to_string(),
                })
                .map_err(|err| {
                    eprintln!("cannot send message ERROR : {err}");
                });
        } else {
            messages
                .send(message::unConnected {
                    ip,
                    stream: stream.clone(),
                })
                .map_err(|err| {
                    eprintln!("cannot send message ERROR : {err}");
                });
            break;
        }
    }
}

fn Server(messages: Receiver<message>) -> () {
    let mut clients = HashMap::new();
    let mut ban_list: HashMap<SocketAddr, Banned> = HashMap::new();
    let BAN_LIMIT: Duration = Duration::from_secs_f32(10.0 * 60.0);
    let RATE_LIMIT: Duration = Duration::from_secs_f32(1.0);
    let STRIKE_LIMIT: u32 = 10;
    loop {
        match messages.recv().unwrap() {
            message::Connected { ip, stream } => {
                clients.insert(
                    ip,
                    Client {
                        strike_count: 0,
                        last_msg_time: SystemTime::now(),
                        stream: stream,
                    },
                );
                ban_list.insert(
                    ip,
                    Banned {
                        isbanned: false,
                        time: SystemTime::now(),
                    },
                );
                println!("{:?}, got connected", ip.to_string());
            }
            message::New_message { ip, str } => {
                println!("new message: {}", str);
                let now = SystemTime::now();
                //update client's info
                let mut update = || {
                    if let Some(client) = clients.get(&ip) {
                        let mut client = client.clone();
                        if now.duration_since(client.last_msg_time).unwrap() < RATE_LIMIT {
                            client.strike_count += 1;
                            if client.strike_count > STRIKE_LIMIT {
                                ban_list.insert(
                                    ip,
                                    Banned {
                                        isbanned: true,
                                        time: now,
                                    },
                                );
                                client.strike_count = 0;
                            }
                        } else {
                            client.strike_count = 0;
                        }
                        client.last_msg_time = now;
                        clients.insert(ip, client.clone());
                        Some(client.stream)
                    } else {
                        None
                    }
                };
                //take care of banned mfs
                let stream = update().unwrap();
                if let Some(Banned { isbanned, time }) = ban_list.get(&ip) {
                    if *isbanned && now.duration_since(*time).unwrap() <= BAN_LIMIT {
                        stream.as_ref().write(b"you are banned\n\r");
                        //stream.as_ref().shutdown(Shutdown::Write); //can't write anymore
                        continue;
                    } else if *isbanned {
                        //unban
                        ban_list.insert(
                            ip,
                            Banned {
                                isbanned: false,
                                time: SystemTime::now(),
                            },
                        );
                    }
                }
                clients.iter().for_each(|(_ip, Client { stream, .. })| {
                    if ip != *_ip {
                        let mut str_out = str.to_string();
                        stream.as_ref().write(str_out.as_bytes());
                    }
                });
            }
            message::unConnected { ip, stream } => {
                println!("{:?}, got disconnected", ip.to_string());
                clients.remove(&ip);
                //stream.shutdown(Both);
            }
        }
    }
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:6969").unwrap();
    let thread_cnt = 4;
    let threadpool = ThreadPool::new(thread_cnt);
    let (sender, receiver) = channel();
    ThreadPool::execute(&threadpool, || Server(receiver));
    for stream in listener.incoming() {
        //stream is multiple ports that accessed the listening socket
        let stream = stream.unwrap();
        //thread::spawn(|| handle_connection(stream));
        let sender = sender.clone();
        let stream = Arc::new(stream);
        ThreadPool::execute(&threadpool, || client(stream, sender));
    }
    //let stream_: TcpStream = listener.incoming().();
    //let (sender_message, client_message) = std::sync::mpsc::channel();
    //let handle = thread::spawn(|| sender_message.send(10).unwrap());
    //handle.join().unwrap();
}

//given a stream http handler
fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream); //socket wrapped with a buffer
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|lines| !lines.is_empty())
        .collect();

    println!("Request: {http_request:#?}");
    let (status_line, filename) = match http_request[0].as_str() {
        "GET /graph_theory HTTP/1.1" => ("HTTP/1.0 200 OK ", "graph_theory.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(10));
            ("HTTP/1.0 200 OK ", "index.html")
        }
        //"Get /sleep HTTP/1.1" =>
        _ => ("HTTP/1.0 404 NOT FOUND", "404.html"),
    };
    let filename = "/Users/jtw/Documents/".to_owned() + filename;
    println!("file path: {}", filename);
    let content = fs::read_to_string(filename).unwrap();
    let content_len = content.len();
    let response = format!("{status_line}\r\nContent-Length: {content_len}\r\n\r\n{content}");

    stream.write_all(response.as_bytes()).unwrap();
}
//fn send_message(){
//  let buf = ""
//  let buf_writer = BufWriter::write_all()
//}
