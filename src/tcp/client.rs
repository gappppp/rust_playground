use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
// use std::io::{self};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use crate::models::models::JsonInfo;

pub fn spawn_client() -> Result<(), Box<dyn std::error::Error>> {
    println!("Client started!");

    let stream  = TcpStream::connect("127.0.0.1:8000");
    if stream.is_err() {
        return Err("Couldn't connect to server...".into());
    }

    let stream = stream.unwrap();
    let _r = stream.set_nonblocking(true);

    if _r.is_err() {
        return Err("The stream could not be set properly".into());
    }
    
    rw_client(stream);

    // //share stream
    // let shared_stream = Arc::new(Mutex::new(stream));
    // let stream_listen = Arc::clone(&shared_stream);
    // let stream_talk = Arc::clone(&shared_stream);

    // //share a shutdown var
    // let shutdown : Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    // let shutdown_listen = Arc::clone(&shutdown);
    // let shutdown_write = Arc::clone(&shutdown);


    // let listener_thread = std::thread::spawn(|| listen_server(stream_listen, shutdown_listen));
    // let talk_thread = std::thread::spawn(|| talk_to_server(stream_talk, shutdown_write));
    
    // listener_thread.join().expect("listener thread panicked");
    // talk_thread.join().expect("talker thread panicked");

    Ok(())
}

// pub fn listen_server(stream: Arc<Mutex<TcpStream>>, shutdown : Arc<AtomicBool>) {
//     let mut buffer = [0; 1024];

//     loop {
//         if shutdown.load(Ordering::SeqCst) {
//             break;
//         }

//         let mut mutex_stream = stream.lock().unwrap();
//         let _r = mutex_stream.set_nonblocking(true);
//         println!("LISTEN");
//         match mutex_stream.read(&mut buffer) {//TODO NOW
//             Ok(0) => {//TODO
//                 println!("Stream closed!");
//                 shutdown.store(true, Ordering::SeqCst);
//                 break;
//             }
//             Ok(n) => {
//                 // println!("! {buffer} !");
//                 let parsed_buffer = String::from_utf8_lossy(&buffer[..n]);
//                 let parsed_buffer = parsed_buffer.trim();
//                 println!("RAW: '{parsed_buffer}' !!!!");
//                 for res in parsed_buffer.split("\n") {//handle all res incoming
//                     println!("[client]: {res}");

//                     if res == "/exit" {
//                         shutdown.store(true, Ordering::SeqCst);
//                         break;
//                     }
//                 }
//             }
//             Err(ref e) if e.kind() == ErrorKind::WouldBlock => {//happens if read has nothing
//                 drop(mutex_stream);//unlock stream for others
//                 // thread::sleep(Duration::from_millis(100));
//                 println!("WAITING SERVER RES...");
//             }
//             Err(e) => {
//                 eprintln!("Client server-listener thread error: {e}");
//                 break;
//             }
//         }
//     }
// }

// pub fn talk_to_server(stream: Arc<Mutex<TcpStream>>, shutdown : Arc<AtomicBool>) {
//     // let res = "Hello. Server!".as_bytes();
//     const RANDOM_RESPONSE: [&str; 4] = ["Bee", "Mooow", "Wof", "/exit"];//TODO: experimental
//     let mut i = 0;//TODO: experimental

//     loop {
//         let res: String = format!("{}\n",RANDOM_RESPONSE[i]);

//         if shutdown.load(Ordering::SeqCst) {
//             break;
//         }

//         if !res.is_empty() {
//             let res_as_bytes = res.as_bytes();
            
//             let mut mutex_stream = stream.lock().unwrap();
//             let _r = mutex_stream.set_nonblocking(true);
//             match mutex_stream.write(res_as_bytes) {
//                 Ok(_) => {
//                     if i < 3 {
//                         i = i + 1;
//                     }
//                     //TODO
//                     thread::sleep(Duration::from_millis(300));
//                 }
//                 Err(ref e) if e.kind() == ErrorKind::WouldBlock => {//happens if read has nothing
//                     drop(mutex_stream);
//                     thread::sleep(Duration::from_millis(150));
//                 }
//                 Err(e) => {
//                     eprintln!("Client server-talker thread error: {e}");
//                     break;
//                 }
//             }
//         }
//     }
// }

pub fn rw_client(mut stream: TcpStream) {
    let mut shutdown = false;
    let _r = stream.set_nonblocking(true);
    
    let mut buffer = [0; 1024];

    // let res = "Hello. Server!".as_bytes();
    const RANDOM_RESPONSE: [&str; 4] = ["Bee", "Mooow", "Wof", "exit"];//TODO: experimental
    let mut i = 0;//TODO: experimental

    while !shutdown {
        //READ ----------------------------------------
        match stream.read(&mut buffer) {
            Ok(0) => {//TODO
                println!("Stream closed!");
                shutdown = true;
            }
            Ok(n) => {
                let parsed_buffer = String::from_utf8_lossy(&buffer[..n]);
                let parsed_buffer = parsed_buffer.trim();

                // println!("RAW: '{parsed_buffer}' !!!!");

                for res in parsed_buffer.split("☃") {//handle all res incoming
                    // println!("RAW: '{res}' !!");
                    let parsed_res = serde_json::from_str(&res);
                    if !parsed_res.is_err() {
                        let parsed_res: JsonInfo = parsed_res.unwrap();
                        println!(
                            "[server]: \n\theader: {}\n\tbody: {}",
                            parsed_res.header,
                            parsed_res.body
                        );
    
                        if parsed_res.header == "exit" {
                            shutdown = true;
                            break;
                        }
                    }
                }
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {//happens if read has nothing
                // thread::sleep(Duration::from_millis(100));
                // println!("WAITING SERVER RES...");
            }
            Err(e) => {
                eprintln!("Client server-listener thread error: {e}");
                shutdown = true;
            }
        }
    
        //WRITE ----------------------------------------
        let res_value: String = format!("{}", RANDOM_RESPONSE[i]);
        let res_value = JsonInfo { header : res_value.to_string(), body: res_value.to_string()};
        let res = serde_json::to_string(&res_value).unwrap();
        let res = format!("{res}☃");
        let res_as_bytes = res.as_bytes();

        if !res.is_empty() {
            match stream.write(res_as_bytes) {
                Ok(_) => {
                    if i < 3 {
                        i = i + 1;
                    }
                    // thread::sleep(Duration::from_millis(300));
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {//happens if read has nothing
                    // thread::sleep(Duration::from_millis(150));
                }
                Err(e) => {
                    eprintln!("Client server-talker thread error: {e}");
                    shutdown = true;
                }
            }
        }
        thread::sleep(Duration::from_millis(200));
    }
}
