use std::fmt::format;
use std::fs::{self};
use std::io::{prelude::*, BufReader};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use Server::ThreadPool;

fn client(stream: Arc<Mutex<TcpStream>>, messages: Sender<&str>) -> () {
    let _ = messages.send("hellow, I am here!\n");
    let mut buffer = vec![0; 1024];
    loop {
        let n = stream.lock().unwrap().read(&mut buffer).unwrap();

        //let ip = stream.peer_addr().unwrap();
        let message_content = "usr : sent {}";
        if n > 0 {
            messages.send(&message_content).map_err(|err| {
                eprintln!("cannot send message ERROR : {err}");
            });
            //write into the Tcp(multi user)
            //but not functioning very well
            stream.lock().unwrap().write(&mut buffer);
        } else {
            println!("n == 0");//printed only when disconnected
            break;
        }
    }
}

fn Server(messages: Receiver<&str>) -> () {
    //TODO: client hashmap
    loop {
        if let Ok(message) = messages.recv() {
            println!("{:?}", message);
        }
    }
}

fn main() {
    let listener = TcpListener::bind("10.0.0.35:7878").unwrap();
    let thread_cnt = 4;
    let threadpool = ThreadPool::new(thread_cnt);
    let (sender, receiver) = channel();
    ThreadPool::execute(&threadpool, || Server(receiver));
    for stream in listener.incoming() {
        //stream is multiple ports that accessed the listening socket
        let stream = stream.unwrap();
        //thread::spawn(|| handle_connection(stream));
        let sender = sender.clone();
        let stream = Arc::new(Mutex::new(stream));
        ThreadPool::execute(&threadpool, || client(stream, sender));
        //handle_connection(stream);
    }
    //let stream_: TcpStream = listener.incoming().();
    //let (sender_message, client_message) = std::sync::mpsc::channel();
    //let handle = thread::spawn(|| sender_message.send(10).unwrap());
    //handle.join().unwrap();
}

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
