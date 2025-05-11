use std::{
    io::{self, ErrorKind, Write},
    net::{TcpListener, TcpStream},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    collections::VecDeque,
    time::Duration,
    task::Poll,
    pin::Pin,
    thread
};

use tokio::{
    io::{AsyncRead, AsyncWriteExt, ReadBuf},
    process::Command as tokio_command
};

use uuid::Uuid;//generate ids for socket representation (used on containers)
use futures::future::poll_fn;

use crate::models::{
    models::*,
    lib::*
};



const SERVER_ADDRESS: &str = "127.0.0.1:8000";
const BUILDER_CONTAINER_NAME: &str = "ruscompy";
const RUNNER_CONTAINER_NAME: &str = "ruruny";
const MAX_CLIENTS: u8 = 10;//accept max n clients


pub fn handle_client(mut stream: TcpStream, id: Uuid) {
    let mut shutdown = false;
    let _r = stream.set_nonblocking(true);
    
    let mut server_res: VecDeque<JsonInfo>= VecDeque::new();

    while !shutdown {
        //READ ----------------------------------------
        loop {
            match read_json_info(&mut stream) {
                Ok(parsed_data) => {
                    println!("[client]: {parsed_data}");

                    if parsed_data.header == "exit" {
                        shutdown = true;
                        break;
                    } else if parsed_data.header == "run&compile" {
                        //tell docker
                        stream = docker_handler(stream, parsed_data.body, id);
                    }
                },
                Err(e) => {
                    if let Some(e) = e.downcast_ref::<io::Error>() {//get original error
                        match e.kind() {
                            ErrorKind::WouldBlock => {//non-block error
                                break;
                            },
                            _ => {//severe error
                                eprintln!("Server error while listening: {e}");
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
        while let Some(res) = server_res.pop_front() {//foreach JsonInfo needed to be sent
            match write_json_info(&mut stream, res) {
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
                                eprintln!("Server error while writing: {e}");
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

fn is_req_shutdown(stream: &mut TcpStream) -> bool {
    match read_json_info(stream) {
        Ok(parsed_data) => {
            parsed_data.header == "exit"
        }
        Err(e) => {
            if let Some(e) = e.downcast_ref::<io::Error>() {//get original error
                match e.kind() {
                    ErrorKind::WouldBlock => {//non-block error
                        false
                    },
                    _ => {//severe error
                        eprintln!("Server error while listening: {e}");
                        true
                    },
                }
            } else {
                false
            }
        }
    }
}

pub fn docker_handler(mut stream: TcpStream, body: String, id: Uuid) -> TcpStream {
    match docker_compile(&mut stream, body, id) {
        Ok(_ok) => {
            match docker_run(&mut stream, id) {
                Ok(_ok) => {
                    // smth
                }
                Err(_err) => {
                    eprintln!("error during run: {_err}");
                }
            }
        }
        Err(_err) => {
            println!("error during compile: {_err}");
        }
    }

    match docker_clean_compile(id) {
        Ok(_ok) => {
            println!("COMPILER container succesfully cleaned!");
        }
        Err(_err) => {
            eprintln!("{_err}");
        }
    }

    match docker_clean_run(id) {
        Ok(_ok) => {
            println!("RUNNER container succesfully cleaned!");
        }
        Err(_err) => {
            eprintln!("{_err}");
        }
    }

    // send exit
    let _o = write_json_info(
        &mut stream,
        JsonInfo::from("exit", "gracefully exit")
    );

    stream
}

pub fn docker_compile(stream: &mut TcpStream, body: String, id: Uuid) -> Result<&str, Box<dyn std::error::Error>> {
    let mut retry_counter: u8 = 0;

    //wait until 'compiler' & 'runner' containers are up
    loop {
        let output = Command::new("docker")
            .args(["ps"])
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);

                if stdout.contains(BUILDER_CONTAINER_NAME) && stdout.contains(BUILDER_CONTAINER_NAME) {
                    break;

                } else {
                    println!(
                        "Container '{}' OR '{}' currently unavailable",
                        BUILDER_CONTAINER_NAME,
                        RUNNER_CONTAINER_NAME
                    );

                    if retry_counter < 10 {
                        println!("Retrying in 3s...");
                        thread::sleep(Duration::from_millis(3000));
                        retry_counter = retry_counter + 1;
                    } else {
                        return Err("Request aborted due to exceed on time limit".into());
                    }
                    
                }
            },
            Err(_err) => {
                println!("{_err}");

                if retry_counter < 10 {
                    println!("Retrying in 3s...");
                    thread::sleep(Duration::from_millis(3000));
                    retry_counter = retry_counter + 1;
                } else {
                    return Err("Request aborted due to exceed on time limit".into());
                }
            },
        }
    }

    if is_req_shutdown(stream) {
        return Err("client requested shutdown prematurely".into());
    }

    let _ = write_json_info(stream, JsonInfo::from(
        "",
        "REQUEST STATUS ----------\nBuilding the file release. This may take a few time..."
    ));

    //insert client body in a new .rs:
    let child = Command::new("docker")
        .args([
            "exec",
            "-i",//necessary when using pipes
            BUILDER_CONTAINER_NAME,
            "sh",
            "-c",
            &format!("cat > ./src/bin/{}", id.to_string()+".rs"),//write smth (cat) inside a file ({})
        ])
        .stdin(Stdio::piped())
        .spawn();

    match child {
        Ok(mut child) => {
            if let Some(mut stdin) = child.stdin.take() {
                match stdin.write_all(body.as_bytes()) {
                    Ok(_ok) => {},
                    Err(_err) => {
                        return Err("ERR_PLAYGROUND_WRITE_CLIENTFILE.RS".into());
                    },
                }
            }
        
            match child.wait() {
                Ok(output) => {
                    if output.success() {
                        println!("Created clientfile.rs succesfully");
                    } else {
                        return Err("ERR_PLAYGROUND_WAIT_CREATE_CLIENTFILE.RS".into());
                    }
                }
                Err(_) => {
                    return Err("ERR_PLAYGROUND_WAIT_CREATE_CLIENTFILE.RS".into());
                }
            }
        }
        Err(_err) => {
            return Err("ERR_PLAYGROUND_CREATE_CLIENTFILE.RS".into());
        }
    }

    if is_req_shutdown(stream) {
        return Err("client requested shutdown prematurely".into());
    }

    //cargo build --release => get exe + more oredered (show compile problems then if ok execute)
    let child = Command::new("docker")
        .args([
            "exec",
            "-i",
            BUILDER_CONTAINER_NAME,
            "sh",
            "-c",
            &format!("cargo build --release --bin {id}")
        ])
        .output();

    match child {
        Ok(output) => {
            let out_str = String::from_utf8_lossy(&output.stderr);

            eprintln!("{out_str}");

            let _r = write_json_info(
                stream, JsonInfo::from("compilation_result", &out_str)
            );

            let output_code_status = output.status.code().unwrap();
            if output_code_status != 0 {
                return Err(
                    format!(
                        "Build failed with status: {}\n{:?}",
                        output_code_status,
                        String::from_utf8_lossy(&output.stderr)
                    ).into()
                );
            }
            
        },
        Err(_err) => {
            return Err("ERR_PLAYGROUND_CARGORUSTC".into());
        }
    }

    //copy .exe from COMPILER to shared VOLUME
    let output = Command::new("docker")
        .args([
            "exec",
            BUILDER_CONTAINER_NAME,
            "sh",
            "-c",
            &format!("cp ./target/release/{id} ../shared_folder/{id}")
        ])
        .output();

    match output {
        Ok(output) => {
            if !output.status.success() {
                eprintln!("Failed to copy file: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(_err) => {
            eprintln!("ERR_PLAYGROUND_CP_EXE");
        }
    }

    Ok("Build created succesfully")
}

pub fn docker_clean_compile(id: Uuid) -> Result<String, Box<dyn std::error::Error>> {
    //WARNING: may not clear everything on container; be sure to run a deep cleaning script every run
    //remove .rs + exe
    docker_rm_file(BUILDER_CONTAINER_NAME, &format!("./src/bin/{id}.rs"))?;
    docker_rm_file(BUILDER_CONTAINER_NAME, &format!("./target/release/{id}"))?;
    docker_rm_file(BUILDER_CONTAINER_NAME, &format!("./target/release/{id}.d"))?;
    
    Ok("Ok".to_string())
}

pub async fn consume_ready<T>(stream: &mut T) -> Result<Vec<u8>, std::io::Error> where T: AsyncRead + Unpin {
    let mut pinned = Pin::new(stream);//target to evalutate
    
    poll_fn(|cx| {
        let mut buf = [0; 1024];
        let mut readbuf = ReadBuf::new(&mut buf);

        match pinned.as_mut().poll_read(cx, &mut readbuf) {//read from target
            Poll::Ready(Ok(())) => {//OK -> return its content
                let filled = readbuf.filled();
                if !filled.is_empty() {
                    return Poll::Ready(Ok(filled.to_vec()));
                } else {
                    return Poll::Ready(Ok("EOF".as_bytes().to_vec()));
                }
            }
            Poll::Ready(Err(e)) => {//Err -> forward it
                eprintln!("Error: {}", e);
                return Poll::Ready(Err(e));
            }
            Poll::Pending => {//Pending -> target does not have produced output yet: return ''
                return Poll::Ready(Ok(Vec::new()));
            }
        }
    }).await
}

pub fn docker_run(mut stream: &mut TcpStream, id: Uuid) -> Result<String, Box<dyn std::error::Error>> {
    let _ = write_json_info(stream, JsonInfo::from("", "EXECUTION ----------"));

    //copy from VOLUME to RUNNER
    let output = Command::new("docker")
        .args([
            "exec",
            RUNNER_CONTAINER_NAME,
            "sh",
            "-c",
            &format!("cp ../shared_folder/{id} ./{id}")
        ])
        .output();

    match output {
        Ok(output) => {
            if !output.status.success() {
                eprintln!("Failed to copy file: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(_err) => {
            eprintln!("ERR_PLAYGROUND_CP_EXE");
        }
    }

    //execute .exe
    if let Ok(async_runtime) = tokio::runtime::Runtime::new() {
        if let Err(err) = async_runtime.block_on(async {//block_on = wait until finish of execution
            let child = tokio_command::new("docker")
                .args(["exec", "-i", RUNNER_CONTAINER_NAME, "sh", "-c", &format!("./{id}")])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();

            let mut shutdown: bool = false;
            let mut server_res: VecDeque<JsonInfo> = VecDeque::new();

            if child.is_err() {
                return Err("ERR_PLAYGROUND_RUN_LAUNCH_EXEC");
            }

            let mut child = child.unwrap();

            let stdin = child.stdin.take();
            let stdout = child.stdout.take();
            let stderr = child.stderr.take();

            if stdin.is_none() || stdout.is_none() || stderr.is_none() {
                return Err("ERR_PLAYGROUND_RUN_TAKE_STDIOS".into());
            }

            let mut stderr = stderr.unwrap();
            let mut stdout = stdout.unwrap();
            let mut stdin = stdin.unwrap();

            while !shutdown {
                //READ STDERR
                loop {
                    match consume_ready(&mut stderr).await {
                        Ok(bytes) if !bytes.is_empty() => {
                            let s = String::from_utf8_lossy(&bytes);

                            if s == "EOF" {
                                println!("stderr pipe reached EOF");
                                shutdown = true;
                                break;
                            }

                            println!("stderr: {s}");
                            server_res.push_back(JsonInfo::from("stderr", &s));
                        }
                        Ok(_) => {//no output available -> skip
                            break;
                        }
                        Err(e) => {
                            eprintln!("Error while reading stderr: {}", e);
                            server_res.push_back(JsonInfo::from("error", &e.to_string()));
                            shutdown = true;
                            break;
                        }
                    }
                }

                //READ STDOUT
                loop {
                    match consume_ready(&mut stdout).await {
                        Ok(bytes) if !bytes.is_empty() => {
                            let s = String::from_utf8_lossy(&bytes);

                            if s == "EOF" {
                                println!("stdout pipe reached EOF");
                                shutdown = true;
                                break;
                            }

                            println!("stdout: {s}");
                            server_res.push_back(JsonInfo::from("stdout", &s));
                        }
                        Ok(_) => {//no output available -> skip
                            break;
                        }
                        Err(e) => {
                            eprintln!("Error while reading stdout: {}", e);
                            server_res.push_back(JsonInfo::from("error", &e.to_string()));
                            shutdown = true;
                            break;
                        }
                    }
                }

                //READ STREAM
                loop {
                    match read_json_info(&mut stream) {
                        Ok(mut client_res) => {
                            if client_res.header == "exit" {
                                shutdown = true;
                                break;

                            } else if client_res.header == "input" {
                                //important since no input is processed if there is not a '\n'
                                if !client_res.body.ends_with('\n') {
                                    client_res.body.push('\n');                                    
                                }

                                if let Err(_err)  = stdin.write_all(&client_res.body.as_bytes()).await {
                                    eprintln!("ERR_PLAYGROUND_FORWARD_STDIN");
                                    server_res.push_back(
                                        JsonInfo::from("error", "ERR_PLAYGROUND_FORWARD_STDIN"
                                    ));
                                    shutdown = true;
                                    break;

                                }
                            }
                        },
                        Err(e) => {
                            if let Some(e) = e.downcast_ref::<io::Error>() {//get original error
                                match e.kind() {
                                    ErrorKind::WouldBlock => {//non-block error
                                        break;
                                    },
                                    _ => {//severe error
                                        eprintln!("Server error while listening: {e}");
                                        server_res.push_back(
                                            JsonInfo::from("error", "Server error while listening"
                                        ));
                                        shutdown = true;
                                        break;

                                    },
                                }
                            } else {//non severe error
                                println!("{e}");
                                server_res.push_back(JsonInfo::from("request_corrupted", ""));   
                            }
                        },
                    }
                }
                
                //WRITE TO CLIENT
                while let Some(res) = server_res.pop_front() {//foreach JsonInfo needed to be sent
                    match write_json_info(&mut stream, res) {
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
                                        return Err("Server error while writing: {e}".into());
                                    },
                                }
                            } else {//non severe error
                                eprintln!("{e}");
                            }
                        },
                    }
                }

                tokio::time::sleep(Duration::from_millis(200)).await;
            }

            // Wait for the child to exit
            if let Err(_err) = child.wait().await {
                return Err("ERR_PLAYGROUND_RUN_CHILD_WAIT".into());
            } else {
                return Ok(());
            }
        }) {
            return Err(err.into());
        }
    } else {
        return Err("ERR_PLAYGROUND_RUN_ASYNC_RT".into());
    }

    Ok("Ok".to_string())
}

pub fn docker_clean_run(id: Uuid) -> Result<String, Box<dyn std::error::Error>> {
    //WARNING: may not clear everything on container; be sure to run a deep cleaning script every run
    //rm exe
    docker_rm_file(RUNNER_CONTAINER_NAME, &format!("{id}"))?;
    docker_rm_file(RUNNER_CONTAINER_NAME, &format!("../shared_folder/{id}"))?;

    Ok("Ok".to_string())
}

pub fn docker_rm_file(container_name: &str, file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let status = Command::new("docker")
        .args(["exec", container_name, "rm", "-f", file_path])
        .status();

    match status {
        Ok(status) => {
            match status.success() {
                true => Ok(format!("file '{container_name}:{file_path}' removed succesfully")),
                false => Err(format!("failed to remove file '{container_name}:{file_path}'").into())
            }
        }
        Err(_err) => {
            Err(_err.into())
        }
    }
}

pub fn spawn_tcp_server() -> Result<(), Box<dyn std::error::Error>> {
    //init general
    let client_accepted: Arc<Mutex<u8>> = Arc::new(Mutex::new(0));

    //init server
    let server = TcpListener::bind(SERVER_ADDRESS);
    if server.is_err() {
        return Err("Fail to bind to adress!".into());
    }
    let server = server.unwrap();

    //loop service
    println!("Server listening on {SERVER_ADDRESS} ...");
    for stream in server.incoming() {
        match stream {
            Ok(mut stream) => {
                let client_accepted = Arc::clone(&client_accepted);
                
                //check client in server
                let mut mutex_client_accepted = client_accepted.lock().unwrap();

                if *mutex_client_accepted < MAX_CLIENTS {
                    //client++ 
                    *mutex_client_accepted = *mutex_client_accepted + 1;
                    drop(mutex_client_accepted);

                    std::thread::spawn(move || handle_client(stream, Uuid::new_v4()));

                    //client--
                    let mut mutex_client_accepted = client_accepted.lock().unwrap();
                    *mutex_client_accepted = *mutex_client_accepted - 1;
                    drop(mutex_client_accepted);

                } else {
                    eprintln!("Max clients number reached, refusing further connections");
                    let _res = write_json_info(
                        &mut stream,
                        JsonInfo::from(
                            "exit",
                            "Max clients number reached, refusing further connections"
                        )
                    );
                    
                    drop(stream);
                }

            }
            Err(e) => {
                eprintln!("Failed to estabilish connection: {e}");
            }
        }
    }

    println!("Server is shutting down!");
    Ok(())
}