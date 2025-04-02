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

    Ok(())
}

pub fn rw_client(mut stream: TcpStream) {
    let mut shutdown = false;
    let _r = stream.set_nonblocking(true);
    
    let mut buffer = [0; 1024];

    // let res = "Hello. Server!".as_bytes();
    // const RANDOM_RESPONSE: [&str; 4] = ["Bee", "Mooow", "Wof", "exit"];//TODO: experimental
    let mut req = JsonInfo::from(
        "run&compile",
        "fn main(){println!(\"FASTARD\")}"
        // r#"use rand::Rng;
        // use std::cmp::Ordering;
        // use std::io;
        
        // fn main() {
        //     println!("Guess the number!");
        
        //     let secret_number = rand::thread_rng().gen_range(1..=100);
        
        //     loop {
        //         println!("Please input your guess.");
        
        //         let mut guess = String::new();
        
        //         io::stdin()
        //             .read_line(&mut guess)
        //             .expect("Failed to read line");
        
        //         let guess: u32 = match guess.trim().parse() {
        //             Ok(num) => num,
        //             Err(_) => continue,
        //         };
        
        //         println!("You guessed: {guess}");
        
        //         match guess.cmp(&secret_number) {
        //             Ordering::Less => println!("Too small!"),
        //             Ordering::Greater => println!("Too big!"),
        //             Ordering::Equal => {
        //                 println!("You win!");
        //                 break;
        //             }
        //         }
        //     }
        // }"#
    );
    // let mut i = 0;//TODO: experimental

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
        // let res_value: String = RANDOM_RESPONSE.to_string();
        // let res_value = JsonInfo::from(res_value.to_string(), res_value.to_string());

        if !req.is_empty() {
            let json_req = serde_json::to_string(&req).unwrap();
            let json_req = format!("{json_req}☃");
            let req_as_bytes = json_req.as_bytes();
            req.clear();

            match stream.write(req_as_bytes) {
                Ok(_) => {
                    // if i < 3 {
                    //     i = i + 1;
                    // }
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
