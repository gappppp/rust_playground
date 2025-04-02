use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use crate::models::models::JsonInfo;

pub fn handle_client(mut stream: TcpStream) {
    let mut shutdown = false;
    let _r = stream.set_nonblocking(true);
    
    let mut res= JsonInfo::new();
    let mut buffer = [0; 1024];

    // let mut i = 1;//TODO

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
                    let parsed_res = serde_json::from_str(&res);
                    if !parsed_res.is_err() {
                        let parsed_res: JsonInfo = parsed_res.unwrap();
                        println!("[client]: {}", parsed_res);
    
                        if parsed_res.header == "exit" {
                            shutdown = true;
                            break;
                        } else if parsed_res.header == "run&compile" {
                            //tell docker
                            stream = docker_run_compile(stream, parsed_res.body);
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
        // let res_value = format!("dummy response n^{i}");
        if !res.is_empty() {
            let res_value = format!("{}☃", serde_json::to_string(&res).unwrap());
            let res_as_bytes = res_value.as_bytes();
            res.clear();//clear = no more write next cycle

            match stream.write(res_as_bytes) {
                Ok(_) => {
                    // i = i + 1;
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {//happens if read has nothing
                    // thread::sleep(Duration::from_millis(150));
                }
                Err(e) => {
                    eprintln!("Server error while writing {e}");
                    shutdown = true;
                }
            }
        }
        
        thread::sleep(Duration::from_millis(200));
    }
}

pub fn docker_run_compile(mut stream: TcpStream, body: String) -> TcpStream {
    const CONTAINER_NAME: &str = "vibrant_blackburn";
    let mut retry_counter: u8 = 0;

    //wait until docker is down
    loop {
        let output = Command::new("docker")
            .args(["ps", "-a", "-f", &format!("name={}", CONTAINER_NAME)])
            .output()
            .expect("Failed to execute Docker command");

        let stdout = String::from_utf8_lossy(&output.stdout);

        if stdout.contains(CONTAINER_NAME) && !stdout.contains("Up") {
            break;

        } else {
            println!("Container '{}' is currenly unavailable", CONTAINER_NAME);
            if retry_counter < 10 {
                println!("Retrying in 3s...");
                thread::sleep(Duration::from_millis(3000));
                retry_counter = retry_counter + 1;
            } else {
                println!("Request aborted due to exceed on time limit");
                return stream;
            }
            
        }
        // else {
        //     eprintln!("Container '{}' not found.", CONTAINER_NAME);
        //     return stream;

        // }
    }

    //boot container
    let output = Command::new("docker")
        .arg("start")
        .arg(CONTAINER_NAME)
        .output()
        .expect("Command for booting the container has failed");

    if output.status.success() {
        println!(
            "Container '{}' started successfully.",
            CONTAINER_NAME
        );
    } else {
        eprintln!(
            "Failed to start the container '{}'. Error: {}",
            CONTAINER_NAME,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    //refresh main.rs with 'body':
    //1 - open file
    let mut open_file_cmd = Command::new("docker")
        .args([
            "exec",
            "-i",
            CONTAINER_NAME,
            "sh",
            "-c",
            "cat > src/main.rs"
        ])
        .stdin(Stdio::piped())
        .spawn()
        .expect("Open stdin for main.rs command has failed");

    //2 - write to file
    if let Some(mut stdin) = open_file_cmd.stdin.take() {
        stdin.write_all(body.as_bytes()).expect("Failed to write to stdin");
    }

    //wait cmd finished
    let output = open_file_cmd.wait().expect("Failed to wait command 'refresh main.rs'");
    if output.success() {
        println!("Inserted client body succesfully!");
    } else {
        println!("Error during client body insertion!");
    }

    //sperimenthal
    let output = Command::new("docker")
        .args([
            "exec",
            "-i",
            CONTAINER_NAME,
            "sh",
            "-c",
            "cat src/main.rs"
        ])
        .output()
        .expect("Failed to read main.rs command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("CAT COMMAND OUTPUT (FROM '{}'):\n{}", CONTAINER_NAME, stdout);
    //end sperimenthal
    
    ///////////////////////////////////////////
    //cargo build --release => get exe + more oredered (show compile problems then if ok execute)
    println!("Building the file release. This may take a few time...");


    let child = Command::new("docker")
        .args([
            "exec",
            "-i",
            CONTAINER_NAME,
            "sh",
            "-c",
            "cargo build --release",
            // "--manifest-path",
            // "torun/Cargo.toml"//TODO:need to be setted
        ])
        .output();

    match child {
        Ok(output) => {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));

            let output_code_status = output.status.code().unwrap();
            if output_code_status != 0 {
                eprintln!("Build failed with status: {}\n{:?}", output_code_status, String::from_utf8_lossy(&output.stderr));
                return stream;//TODO add before a msg
            }
            
        },
        Err(_err) => {
            eprintln!("ERR_PLAYGROUND_CARGOBUILD");
            return stream;//TODO add before a msg
        }
    }

    //execute .exe
    println!("Build successful. Running now the build...");

    let child = Command::new("docker")
        .args(["exec", "-i", CONTAINER_NAME, "sh", "-c", "./target/release/compiler_runner_test"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    match child {
        Err(_err) => {
            eprintln!("ERR_PLAYGROUND_CARGORUN_SPAWN");
            eprintln!("{_err}");
        }
        Ok(mut child) => {
            let mut stdin = child.stdin.take().expect("ERR_PLAYGROUND_RUN_TAKE_STDIN");
            let mut stdout = child.stdout.take().expect("ERR_PLAYGROUND_RUN_TAKE_STDOUT");
            let mut stderr = child.stderr.take().expect("ERR_PLAYGROUND_RUN_TAKE_STDERR");

            let mut stdout_buff = [0u8; 1024];
            let mut stderr_buff = [0u8; 1024];

            // println!("--- START CARGO RUN ---");

            //loop-read stdout & loop-write stdin
            let stdio_t =thread::spawn(move || {
                let mut max: u8 = 100;//TODO
                let mut min: u8 = 1;
                let mut curr: u8 = 50;
                let mut curr_str = curr.to_string()+"\n";

                loop {
                    match stdout.read(&mut stdout_buff) {
                        Ok(0) => {
                            break;//cmd terminated
                        }
                        Err(_e) => {//report err
                            eprintln!("Error reading from stdout: {:?}", _e);
                            break;
                        }
                        Ok(_n) => {
                            let readed = String::from_utf8_lossy(&mut stdout_buff);
                            let readed = readed.trim();
                            println!("{readed}");
                
                            //TODO: logic to remove
                            if readed.contains("big") {
                                max = curr;
                
                                if min == max + 1 {
                                    curr = min;
                                } else {
                                    curr = (max+min)/2;
                                }
                
                                curr_str = curr.to_string()+"\n";
                            }
                            if readed.contains("small") {
                                min = curr;
                
                                if min == max + 1 {
                                    curr = max;
                                } else {
                                    curr = (max+min)/2;
                                }
                
                                curr_str = curr.to_string()+"\n";
                            }
                            //end of TODO
                
                            if readed.contains("Please input your guess") {//TODO change logic
                                //N.B.: io::stdin() EXPECTS a \n after input
                                if let Err(e) = stdin.write_all(curr_str.as_bytes()) {
                                    eprintln!("Failed to write to stdin: {:?}", e);
                                    break;
                                }
                                if let Err(e) = stdin.flush() {
                                    eprintln!("Failed to flush stdin: {:?}", e);
                                    break;
                                }
                            }
                        }
                    }

                    thread::sleep(Duration::from_millis(200));
                }
            });

            //loop-read stderr
            let stderr_t = thread::spawn(move || {
                loop {
                    match stderr.read(&mut stderr_buff) {
                        Ok(0) => {
                            break;
                        }
                        Err(_e) => {
                            eprintln!("Error reading from stdout: {:?}", _e);
                            break;
                        }
                        Ok(_n) => {
                            eprintln!("{}", String::from_utf8_lossy(&mut stderr_buff));
                        }
                    }

                    thread::sleep(Duration::from_millis(200));
                }
            });

            //'wait_with_output' = wait until cmd completed (IMPORTANT)
            let final_output = child.wait_with_output().expect("ERR_PLAYGROUND_RUN_WAIT_END");
            stdio_t.join().expect("ERR_PLAYGROUND_RUN_WAIT_STDIO");
            stderr_t.join().expect("ERR_PLAYGROUND_RUN_WAIT_STDERR");

            //print exit stats
            match final_output.status.success() {
                true => println!("Process completed successfully!"),
                false => println!("Process failed with status: {:?}", final_output.status.code().unwrap()),
            }

            //eventually print final stdout
            let final_stdout = String::from_utf8_lossy(&final_output.stdout);
            if !final_stdout.is_empty() {
                println!("--- STDOUT ---\n{final_stdout}");
            }

            //eventually print final stderr
            let final_stderr = String::from_utf8_lossy(&final_output.stderr);
            if !final_stderr.is_empty() {
                println!("--- STDERR ---\n{final_stderr}");
            }
        }
    }
    ///////////////////////////////////////////

    // //end: wait program to exit
    // cargo_run_cmd.wait().expect("Failed to wait on child process");
    let _o = stream.write(//TODO: send exit (temp)
        format!(
            "{}☃",
            serde_json::to_string(&JsonInfo::from("exit", "")).unwrap()
        ).as_bytes()
    );

    stream
}

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
                std::thread::spawn(|| handle_client(stream));
            }
            Err(e) => {
                eprintln!("Failed to estabilish connection: {e}");
            }
        }
    }

    Ok(())
}