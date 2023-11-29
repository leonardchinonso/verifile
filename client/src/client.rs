use std::io::{Read, Write};
use std::net::{TcpStream};
use std::fs::File;
use common::SERVER_ADDRESS;

pub struct Client {
    file_names: Vec<String>
}

impl Client {
    pub fn new(file_names: Vec<String>) -> Self {
        Self{
            file_names,
        }
    }

    pub fn process(&self) {
        // Write and send
        let mut stream = TcpStream::connect(SERVER_ADDRESS).expect("Failed to connect to server.");
        let mut file = File::open(&self.file_names[0]).unwrap();
        let mut buffer_send = Vec::new();
        file.read_to_end(&mut buffer_send).unwrap();

        stream.write_all(&buffer_send).expect("Failed to send file to server");

        let mut buffer = [0; 512];
        let bytes_read = stream.read(&mut buffer).expect("Failed to read from server");
        let response = String::from_utf8_lossy(&buffer[..bytes_read]);

        println!("Message from server: {}", response);
    }
}

