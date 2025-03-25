use std::io::{self};
mod tcp;
mod models;

fn main() {
    //select role
    let mut choice : u32;

    println!("Insert role:\n0 -> Server\n1 -> Client");

    loop {
        let mut choice_str = String::new();
        io::stdin().read_line(&mut choice_str).expect("Failed to recieve role");
        choice =  match choice_str.trim().parse() {
            Ok(num) => num,
            Err(_e) => u32::MAX
        };


        if choice <= 1 {
            break;
        } else {
            println!("Select a valid role!");
        }
    };

    //call role methods
    if choice == 0 {
        match tcp::server::spawn_tcp_server() {
            Ok(_) => {
                println!("Server exited succesfully!");
            }
            Err(e) => {
                println!("'Server' exit status: {e}");
            }
        }
    } else {
        match tcp::client::spawn_client() {
            Ok(_) => {
                println!("Client exited succesfully!");
            }
            Err(e) => {
                println!("'Client' exit status: {e}");
            }
        }
    }
}