use std::{io::{Read, Write}, net::TcpStream};

use crate::models::models::JsonInfo;

const MAX_READ_BUFF_LEN : usize = 30000;
const MSG_INDEX_LENGTH: usize = 4;


pub fn read_json_info(stream: &mut TcpStream) -> Result<JsonInfo, Box<dyn std::error::Error>> {
    //read a u32 that indicates msg len sent
    let mut buffer_len: [u8; MSG_INDEX_LENGTH] = [0u8; MSG_INDEX_LENGTH];

    match stream.read_exact(&mut buffer_len) {// read size first
        Ok(_ok) => {
            let len  = u32::from_be_bytes(buffer_len) as usize;
            if len > MAX_READ_BUFF_LEN {
                return Err("data too large to read".into());
            }

            let mut temp_buff = vec![0u8; len];

            match stream.read_exact(&mut temp_buff) {//read n (= size) bytes
                Ok(_ok) => {
                    let parsed_data = String::from_utf8_lossy(&temp_buff);
                    let parsed_data = parsed_data.trim();
                    let parsed_data: Result<JsonInfo, serde_json::Error> = serde_json::
                        from_str(parsed_data);
                    
                    match parsed_data {//try parse in JsonInfo
                        Ok(parsed_data) => Ok(parsed_data),
                        Err(_err) => Err("Failed to Deserialize on read".into()),
                    }

                },
                // Err(ref e) if e.kind() == ErrorKind::WouldBlock => {//happens if read has nothing
                //     Err(e.into())
                // },
                Err(e) => {
                    Err(e.into())
                },
            }
        },
        // Err(ref e) if e.kind() == ErrorKind::WouldBlock => {//happens if read has nothing
        //     // thread::sleep(Duration::from_millis(100));
        //     break;
        // },
        Err(e) => {
            Err(e.into())
        },
    }
}

pub fn write_json_info(stream: &mut TcpStream, data: JsonInfo) -> Result<(), Box<dyn std::error::Error>> {
    let _return : Result<(), Box<dyn std::error::Error>>;
    
    match serde_json::to_string(&data) {//try deserialize
        Ok(parsed_data) => {
            let parsed_data_as_bytes = parsed_data.as_bytes();//get data into bytes
            let data_len = parsed_data_as_bytes.len();//calc data length
            let data_len_len = data_len as u32;
            
            //compose single message like {data_len: u32, data: String}
            //N.B.: data_len necessary to determine start/end of msgs
            let mut msg = Vec::with_capacity(MSG_INDEX_LENGTH + data_len);
            msg.extend_from_slice(&data_len_len.to_be_bytes());
            msg.extend_from_slice(parsed_data_as_bytes);
            
            match stream.write_all(&msg) {
                Ok(_ok) => _return = Ok(()),
                // Err(ref e) if e.kind() == ErrorKind::WouldBlock => {//happens if read has nothing
                //     // thread::sleep(Duration::from_millis(150));
                // }
                Err(e) => _return = Err(e.into()),
            }

            let _f = stream.flush();//no need to handle
        },
        Err(_err) => {
            _return = Err("Failed to Deserialize on read".into());
        },
    }

    _return
}