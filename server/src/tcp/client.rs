use std::collections::VecDeque;
use std::io::{self, ErrorKind};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use crate::models::lib::*;
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
    //init
    let mut shutdown = false;

    let mut req: VecDeque<JsonInfo> = VecDeque::new();
    req.push_back(JsonInfo::from(
        "run&compile",
        "fn main(){println!(\"CAGATI ADDOSSO . io!\")}"
    ));
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
    // req.push_back(JsonInfo::from("dummy", ""));

    //streaming loop
    while !shutdown {
        //READ ----------------------------------------
        loop {
            match read_json_info(&mut stream) {
                Ok(parsed_data) => {
                    println!("[server]: {parsed_data}");

                    if parsed_data.header == "exit" {
                        shutdown = true;
                        break;
                    }
                },
                Err(e) => {
                    if let Some(e) = e.downcast_ref::<io::Error>() {//get original error
                        match e.kind() {
                            ErrorKind::WouldBlock => {//non-block error
                                break;
                            },
                            _ => {//severe error
                                eprintln!("Client error while listening: {e}");
                                shutdown = true;
                                break;
                            },
                        }
                    } else {//non severe error
                        eprintln!("{e}");
                    }
                },
            }
        }
    
        //WRITE ----------------------------------------
        while let Some(r) = req.pop_front() {//foreach JsonInfo needed to be sent
            match write_json_info(&mut stream, r) {
                Ok(_ok) => {
                    // smth
                },
                Err(e) => {
                    if let Some(e) = e.downcast_ref::<io::Error>() {//get original error
                        match e.kind() {
                            ErrorKind::WouldBlock => {//non-block error
                                // smth
                            },
                            _ => {//severe error
                                eprintln!("Client error while writing: {e}");
                                shutdown = true;
                                break;
                            },
                        }
                    } else {//non severe error
                        eprintln!("{e}");
                    }
                },
            }
        }

        thread::sleep(Duration::from_millis(200));
    }
}

//TODO: add rsx client
