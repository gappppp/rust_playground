use std::{
    io::{self, ErrorKind},
    net::TcpStream
};

use crate::tcp::jsoninfo::*;



pub struct TcpClient {
    pub stream: Option<TcpStream>,
}

impl TcpClient {
    pub fn init_as_none() -> Self {
        TcpClient { stream: None }
    }

    pub fn shutdown(&mut self) {
        if let Some(stream) = self.stream.as_mut() {
            let _ = write_json_info(stream, JsonInfo::from("shutdown", ""));
            let _ = stream.shutdown(std::net::Shutdown::Both);
        }
        self.stream = None;
    }

    pub fn spawn(addr: &str) -> Result<Self, ()> {
        match Self::spawn_stream(addr) {
            Ok(stream) => Ok(TcpClient { stream: Some(stream) }),
            Err(err) => Err(err),
        }
    }

    fn spawn_stream(addr: &str) -> Result<TcpStream, ()> {
        let stream = TcpStream::connect(addr);

        if stream.is_err() {
            return Err(());
        }
        
        let stream = stream.expect(format!("Error: Trying to connect to {addr} failed").as_str());

        if stream.set_nonblocking(true).is_err() {
            return Err(());
        }

        Ok(stream)
    }

    pub fn read(&mut self) -> Result<Option<JsonInfo>, Box<dyn std::error::Error>> {
        if let Some(tcpclient) = self.stream.as_mut() {
            if let Ok(mut stream) = tcpclient.try_clone() {
                match read_json_info(&mut stream) {
                    Ok(info) => {
                        match info.header.as_str() {
                            "request_corrupted" => Ok(None),
                            _ => Ok(Some(info)),
                        }
                    },
                    Err(e) => {
                        if let Some(e) = e.downcast_ref::<io::Error>() {//get original error
                            match e.kind() {
                                ErrorKind::WouldBlock => {//non-block error
                                    Ok(None)
                                },
                                _ => {//severe error
                                    Err(format!("Client error while listening: {:?}", e).into())
                                },
                            }
                        } else {//non severe error
                            Ok(None)
                        }
                    }
                }
            } else {
                Err(format!("TcpClient could not be accessed").into())
            }
        } else {
            Err(format!("TcpClient was not initialized").into())
        }
    }

    pub fn send_run_compile_req(&mut self, req: String) {
        if let Some(stream) = self.stream.as_mut() {
            let _r = write_json_info(
                stream,
                JsonInfo::from("run&compile", &req
            ));
        }
    }

    pub fn send_input_req(&mut self, req: String) {
        if let Some(stream) = self.stream.as_mut() {
            let _ = write_json_info(
                stream,
                JsonInfo::from("input", &req)
            );
        }
    }
}