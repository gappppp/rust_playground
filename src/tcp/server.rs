use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;
use crate::models::models::JsonInfo;

pub fn handle_client2(mut stream: TcpStream) {
    let mut shutdown = false;
    let _r = stream.set_nonblocking(true);
    
    let mut buffer = [0; 1024];

    let mut i = 1;//TODO

    while !shutdown {
        //READ ----------------------------------------
        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("handle client thread closing...");
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
                            "[client]: \n\theader: {}\n\tbody: {}",
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
            }
            Err(e) => {
                eprintln!("Server error while listening: {e}");
                shutdown = true;
            }
        }
        
        //WRITE ----------------------------------------
        let res_value = format!("dummy response n^{i}");
        let res_value = JsonInfo { header : res_value.to_string(), body: res_value.to_string()};
        let res = serde_json::to_string(&res_value).unwrap();
        let res = format!("{res}☃");
        let res_as_bytes = res.as_bytes();

        match stream.write(res_as_bytes) {
            Ok(_) => {
                i = i + 1;
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {//happens if read has nothing
                // thread::sleep(Duration::from_millis(150));
            }
            Err(e) => {
                eprintln!("Server error while writing {e}");
                shutdown = true;
            }
        }
        thread::sleep(Duration::from_millis(200));
    }
}

// pub fn handle_client(mut stream: TcpStream) {
//     let mut shutdown = false;
//     let _r = stream.set_nonblocking(true);
//     let mut buffer = [0; 1024];

//     let mut i = 1;//TODO

//     while !shutdown {
//         //READ REQ CLIENT ----------------
//         match stream.read(&mut buffer) {
//             Ok(0) => {//TODO
//                 println!("handle client thread closing...");
//                 shutdown = true;
//             }
//             Ok(n) => {
//                 let req = String::from_utf8_lossy(&buffer[..n]);
//                 let req = req.trim();
//                 println!("[client]: {req}");

//                 if req == "/exit" {
//                     println!("[!]: elaborating client request to close stream");
//                     shutdown = true;
//                 }
//             }
//             Err(ref e) if e.kind() == ErrorKind::WouldBlock => {//happens if read has nothing
//                 // thread::sleep(Duration::from_millis(200));
//             }
//             Err(e) => {
//                 eprintln!("Server error while listening: {e}");
//                 shutdown = true;
//             }
//         }//END OF READ REQ CLIENT ----------------

//         //GENERATE RESPONSE ---------------------
//         let res = format!("dummy response n^{i}\n");
//         // println!("GENerate response: {res}");
//         let res = res.as_str().as_bytes();

//         match stream.write(res) {
//             Ok(_) => {
//                 i = i + 1;
//                 //TODO
//             }
//             Err(ref e) if e.kind() == ErrorKind::WouldBlock => {//happens if read has nothing
//                 // thread::sleep(Duration::from_millis(150));
//             }
//             Err(e) => {
//                 eprintln!("Server error while writing {e}");
//                 shutdown = true;
//             }
//         }
//         //END OF GENERATE RESPONSE ---------------------

//         thread::sleep(Duration::from_millis(200));
//     }

//     println!("[!]: stream succesfully closed!");
// }

pub fn spawn_tcp_server() -> Result<(), Box<dyn std::error::Error>> {
    //init server
    let server = TcpListener::bind("127.0.0.1:8000");
    if server.is_err() {
        return Err("Fail to bind to adress!".into());
    }
    let server = server.unwrap();

    //loop service
    println!("Server listening on 127.0.0.1:8000 ...");
    for stream in server.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(|| handle_client2(stream));
            }
            Err(e) => {
                eprintln!("Failed to estabilish connection: {e}");
            }
        }
    }

    Ok(())
}