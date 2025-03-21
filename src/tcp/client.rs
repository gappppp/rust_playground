use std::io::{ErrorKind, Read, Write};
// use std::io::{self};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub fn spawn_client() -> Result<(), Box<dyn std::error::Error>> {
    println!("Client started!");

    let stream  = TcpStream::connect("127.0.0.1:8000");
    if stream.is_err() {
        return Err("Couldn't connect to server...".into());
    }
    println!("client init succesfull");
    let stream = stream.unwrap();
    let _r = stream.set_nonblocking(true);

    if _r.is_err() {
        return Err("The stream could not be set properly".into());
    }
    
    //share stream
    let shared_stream = Arc::new(Mutex::new(stream));
    let stream_listen = Arc::clone(&shared_stream);
    let stream_talk = Arc::clone(&shared_stream);

    let listener_thread = std::thread::spawn(|| listen_server(stream_listen));
    let talk_thread = std::thread::spawn(|| talk_to_server(stream_talk));
    println!("launched theards!");
    listener_thread.join().expect("listener thread panicked");
    talk_thread.join().expect("talker thread panicked");

    Ok(())
}

pub fn listen_server(stream: Arc<Mutex<TcpStream>>) {
    let mut buffer = [0; 1024];

    loop {
        let mut mutex_stream = stream.lock().unwrap();
        println!("Listen");

        match mutex_stream.read(&mut buffer) {
            Ok(0) => {//TODO
                println!("Stream closed!");
                break;
            }
            Ok(n) => {
                let req = String::from_utf8_lossy(&buffer[..n]);
                println!("[client] readed: {req}");

                if req == "/exit" {
                    break;
                }
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {//happens if read has nothing
                drop(mutex_stream);//unlock stream for others
                thread::sleep(Duration::from_millis(200));
            }
            Err(e) => {
                eprintln!("Client server-listener thread error: {e}");
            }
        }
    }
}

pub fn talk_to_server(stream: Arc<Mutex<TcpStream>>) {
    let res = "Hello. Server!".as_bytes();

    loop {
        let mut mutex_stream = stream.lock().unwrap();
        println!("Talk");

        match mutex_stream.write(res) {
            Ok(_) => {
                break;
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {//happens if read has nothing
                drop(mutex_stream);
                thread::sleep(Duration::from_millis(150));
            }
            Err(e) => {
                eprintln!("Client server-talker thread error: {e}");
            }
        }
    }
}