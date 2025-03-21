use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

pub fn handle_client(mut stream: TcpStream) {
    //read client request
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).expect("Failed to read client request");

    //elaborate
    let req = String::from_utf8_lossy(&buffer[..]);
    println!("req: {req}");

    //write client response
    let res = "Hello. Client!".as_bytes();
    stream.write(res).expect("Failed to write resposnse");
}

pub fn spawn_tcp_server() -> Result<(), Box<dyn std::error::Error>> {
    //init server
    let server = TcpListener::bind("127.0.0.1:8000");
    if server.is_err() {
        return Err("Fail to bind to adress!".into());
    }
    let server = server.unwrap();

    //loop service
    println!("Server listening on 127.0.0.1:8000");
    for stream in server.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(|| handle_client(stream));
            }
            Err(e) => {
                eprintln!("Failed to estabilish connection: {e}");
            }
        }
    }

    Ok(())
}